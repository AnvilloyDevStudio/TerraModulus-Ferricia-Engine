/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use crate::mui::ogl::{buf_obj_with_data, compile_shader, draw_arrays, draw_elements, gen_buf_obj, gen_buf_objs, get_uniform_location, new_shader_program, use_program, use_texture_2d, use_uniform_mat_4, use_vao, vert_attr, vert_attr_arr, with_new_vert_arr, GLHandle, NumType, ShaderType, VertexAttrVariant};
use crate::mui::window::WindowHandle;
use crate::FerriciaResult;
use gl::{BindTexture, GenTextures, GenerateMipmap, TexImage2D, TexParameteri, ARRAY_BUFFER, CLAMP_TO_EDGE, ELEMENT_ARRAY_BUFFER, LINES, NEAREST, NEAREST_MIPMAP_LINEAR, RGBA, STATIC_DRAW, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, TRIANGLES, UNSIGNED_BYTE};
use image::imageops::flip_vertical;
use image::ImageReader;
use nalgebra_glm::{identity, ortho, scaling, translate, vec3, TMat4};
use ordermap::OrderSet;
use sdl3::pixels::Color;
use std::borrow::Cow;
use std::cell::Cell;
use std::fs::read_to_string;
use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::{Arc, LazyLock};

static IDENT_MAT_4: LazyLock<TMat4<f32>> = LazyLock::new(identity);

pub(crate) struct CanvasHandle {
	/// Size of Canvas in pixels
	size: (u32, u32),
	ortho_proj_mat: TMat4<f32>,
	// drawable_sets: HashMap<OpaqueId, DrawableSet>,
	used_program: Cell<u32>,
	/// DO NOT MUTATE
	gl_handle: Arc<GLHandle>,
}

impl CanvasHandle {
	pub(crate) fn new(window_handle: &WindowHandle) -> Self {
		let size = window_handle.window_size_in_pixels();
		let gl_handle = window_handle.gl_handle().clone();
		Self {
			ortho_proj_mat: ortho_proj_mat(size),
			size,
			gl_handle,
			used_program: Cell::new(0),
			// drawable_sets: HashMap::new(),
		}
	}

	// pub(crate) fn new_drawable_set(&mut self, prim: impl RenderPrimitive + 'static) -> &DrawableSet {
	// 	let set = DrawableSet::new(prim);
	// 	let id = set.id;
	// 	if let Some(v) = self.drawable_sets.insert(set.id, set) {
	// 		panic!("{:?} should be unique", v.id)
	// 	}
	// 	self.drawable_sets.get(&id).expect("should exist")
	// }

	pub(crate) fn load_image(&self, path: String) -> u32 {
		let img = ImageReader::open(path)
			.expect("Cannot open image")
			.decode()
			.expect("Cannot decode image")
			.into_rgba8();
		// Image coordinates have a difference direction as OpenGL texture coordinates.
		flip_vertical(&img);
		let mut id = MaybeUninit::uninit();
		unsafe { GenTextures(1, id.as_mut_ptr()); }
		let id = unsafe { id.assume_init() };
		unsafe { BindTexture(TEXTURE_2D, id); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST_MIPMAP_LINEAR as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _); }
		unsafe {
			TexImage2D(
				TEXTURE_2D,
				0,
				RGBA as _,
				img.width() as _,
				img.height() as _,
				0,
				RGBA,
				UNSIGNED_BYTE,
				img.as_ptr() as *const _
			);
		}
		unsafe { GenerateMipmap(TEXTURE_2D) }
		id
	}

	pub(crate) fn refresh_canvas_size(&mut self, width: u32, height: u32) {
		self.size = (width, height);
		self.ortho_proj_mat = ortho_proj_mat(self.size);
	}

	pub(crate) fn draw_gui(&self, set: &DrawableSet, program: &impl GuiProgram, texture: Option<u32>) {
		if self.used_program.get() != program.id() {
			program.apply();
			self.used_program.set(program.id());
		}

		if let Some(v) = texture {
			use_texture_2d(v);
		}
		
		set.prim.apply_vao();
		let context = DrawingContext { window_size: &self.size };
		program.uniform(&self.ortho_proj_mat, set, context);
		set.prim.draw();
	}
}

struct DrawingContext<'a> {
	window_size: &'a (u32, u32),
}

/// Usage: `unsafe { UniformMatrix4fv(0, 1, FALSE, ortho.as_ptr()) }`
///
/// This may be an identity matrix if no model/view matrix is supplied.
fn ortho_proj_mat(size: (u32, u32)) -> TMat4<f32> {
	let (width, height) = size;
	ortho::<f32>(0., width as _, 0., height as _, -1., 1.)
}

fn compile_shader_from(kind: ShaderType, path: String) -> FerriciaResult<u32> {
	Ok(compile_shader(read_to_string(path).expect("Cannot read the file"), kind)?)
}

#[allow(private_interfaces)]
pub(crate) trait GuiProgram {
	fn id(&self) -> u32;

	fn apply(&self);

	fn uniform(&self, proj: &TMat4<f32>, set: &DrawableSet, drawing_context: DrawingContext);
}

pub(crate) struct GeoProgram {
	id: u32,
	model_pos: u32,
	projection_pos: u32,
}

impl GeoProgram {
	pub(crate) fn new(vsh: String, fsh: String) -> FerriciaResult<Self> {
		let id = new_shader_program([
			compile_shader_from(ShaderType::Vertex, vsh)?,
			compile_shader_from(ShaderType::Fragment, fsh)?,
		]);
		Ok(Self {
			model_pos: get_uniform_location(id, "model"),
			projection_pos: get_uniform_location(id, "projection"),
			id,
		})
	}
}

#[allow(private_interfaces)]
impl GuiProgram for GeoProgram {
	fn id(&self) -> u32 {
		self.id
	}

	#[inline]
	fn apply(&self) {
		use_program(self.id);
	}

	fn uniform(&self, proj: &TMat4<f32>, set: &DrawableSet, drawing_context: DrawingContext) {
		use_uniform_mat_4(self.projection_pos, proj);
		let model = set.eval_model_mat(&drawing_context);
		use_uniform_mat_4(self.model_pos, model.as_ref());
	}
}

pub(crate) struct TexProgram {
	id: u32,
	model_pos: u32,
	projection_pos: u32,
	filter_pos: u32,
}

impl TexProgram {
	pub(crate) fn new(vsh: String, fsh: String) -> FerriciaResult<Self> {
		let id = new_shader_program([
			compile_shader_from(ShaderType::Vertex, vsh)?,
			compile_shader_from(ShaderType::Fragment, fsh)?,
		]);
		Ok(Self {
			model_pos: get_uniform_location(id, "model"),
			projection_pos: get_uniform_location(id, "projection"),
			filter_pos: get_uniform_location(id, "filter"),
			id,
		})
	}
}

#[allow(private_interfaces)]
impl GuiProgram for TexProgram {
	fn id(&self) -> u32 {
		self.id
	}

	#[inline]
	fn apply(&self) {
		use_program(self.id);
	}

	fn uniform(&self, proj: &TMat4<f32>, set: &DrawableSet, drawing_context: DrawingContext) {
		use_uniform_mat_4(self.projection_pos, proj);
		let model = set.eval_model_mat(&drawing_context);
		use_uniform_mat_4(self.model_pos, model.as_ref());
		let filter = set.eval_filter_mat(&drawing_context);
		use_uniform_mat_4(self.filter_pos, filter.as_ref());
	}
}

// /// All the state data visible to the rendering (main) thread to be processed.
// /// All data access must be done using `Mutex` locks; should not be accessed twice
// /// at the same time in the same thread, as it might panic.
// struct RenderingState {
// 	lock: Mutex<()>,
// }
//
// impl RenderingState {
// 	fn new() -> Self {
// 		Self {
// 			lock: Mutex::new(()),
// 		}
// 	}
// }
//
// /// A sequence of links to `DrawableSet` that are drawn by this instance.
// pub(crate) struct DrawingLinks<'a> {
// 	id: OpaqueId,
// 	links: HashMap<OpaqueId, DrawingNode<'a>>,
// }
//
// impl DrawingLinks<'_> {
// 	fn new() -> Self {
// 		static COUNTER: AtomicUsize = AtomicUsize::new(0);
// 		Self {
// 			id: OpaqueId::new(&COUNTER),
// 			links: HashMap::new(),
// 		}
// 	}
//
// 	fn add_link(node: DrawingNode) {
//
// 	}
// }
//
// enum DrawingNode<'a> {
// 	Links(&'a DrawingLinks<'a>),
// 	Set(&'a DrawableSet),
// }
//
// impl<'a> DrawingNode<'a> {
// 	fn id(&self) -> OpaqueId {
// 		match self {
// 			DrawingNode::Links(v) => v.id,
// 			DrawingNode::Set(v) => v.id,
// 		}
// 	}
// }

/// A set of data that is completely drawable for an instance with all the information available.
/// 
/// The functions of model and filter additions and removals cannot be made generalized by pointer
/// arithmetic operations due to the current limitation of pointers.
/// See [Rust RFC 2580](https://rust-lang.github.io/rfcs/2580-ptr-meta.html) for details.
// #[derive(Getters)]
pub(crate) struct DrawableSet<'a> {
	// #[getset(get = "pub")]
	// id: OpaqueId,
	prim: Box<dyn RenderPrimitive>,
	models: OrderSet<&'a dyn PrimModelTransform>,
	/// Usable only for [`Mesh`]es.
	filters: OrderSet<&'a dyn PrimColorFilter>,
	// _pin: PhantomPinned,
}

impl<'a> DrawableSet<'a> {
	pub(crate) fn new(prim: impl RenderPrimitive + 'static) -> Self {
		// static COUNTER: AtomicUsize = AtomicUsize::new(0);
		Self {
			// id: OpaqueId::new(&COUNTER),
			prim: Box::new(prim),
			models: OrderSet::new(),
			filters: OrderSet::new(),
			// _pin: PhantomPinned,
		}
	}

	/// Requires careful management
	pub(crate) fn prim<T: RenderPrimitive>(&mut self) -> &mut T {
		unsafe { &mut *(self.prim.as_mut() as *mut dyn RenderPrimitive as *mut T) }
	}

	pub(crate) fn add_model_transform(&mut self, transform: &'a dyn PrimModelTransform) {
		self.models.insert(transform);
	}

	pub(crate) fn remove_model_transform(&mut self, transform: &'a dyn PrimModelTransform) {
		self.models.remove(&transform);
	}

	pub(crate) fn add_filter_transform(&mut self, filter: &'a dyn PrimColorFilter) {
		self.filters.insert(filter);
	}

	pub(crate) fn remove_filter_transform(&mut self, filter: &'a dyn PrimColorFilter) {
		self.filters.remove(&filter);
	}

	fn eval_model_mat(&self, drawing_context: &DrawingContext) -> Cow<TMat4<f32>> {
		if self.models.is_empty() {
			Cow::Borrowed(&*IDENT_MAT_4)
		} else {
			let mut it = self.models.iter();
			let first = it.next().unwrap();
			Cow::Owned(it.fold(first.model_matrix(drawing_context), |m1, m2| m2.model_matrix(drawing_context) * m1))
		}
	}

	fn eval_filter_mat(&self, drawing_context: &DrawingContext) -> Cow<TMat4<f32>> {
		if self.models.is_empty() {
			Cow::Borrowed(&*IDENT_MAT_4)
		} else {
			let mut it = self.filters.iter();
			let first = it.next().unwrap();
			Cow::Owned(it.fold(first.filter_matrix(drawing_context), |m1, m2| m2.filter_matrix(drawing_context) * m1))
		}
	}
}

pub(crate) trait RenderPrimitive {
	fn vao(&self) -> u32;

	#[inline]
	fn apply_vao(&self) {
		use_vao(self.vao());
	}

	fn draw(&self);
}

/// All `Geom`s take coordinates as screen coordinates.
trait Geom : RenderPrimitive {

}

/// Linear Geom with only two points and one color. This uses `LINES`.
pub(crate) struct SimpleLineGeom {
	vao: u32,
	vbo: u32,
}

impl SimpleLineGeom {
	const NUM_VERTICES: u32 = 2;
	pub(crate) fn new(points: [(f32, f32); 2], color: Color) -> Self {
		let vao = with_new_vert_arr();
		let vbo = gen_buf_obj();
		let vertices = [
			points[0].0, points[0].1,
			points[1].0, points[1].1,
		];
		buf_obj_with_data(ARRAY_BUFFER, vbo, &vertices, STATIC_DRAW);
		vert_attr_arr(0, 2, NumType::Float, 2, 0); // Position
		vert_attr(1, VertexAttrVariant::UbyteNorm4.call(color.rgba())); // Color
		Self { vao, vbo } // Note: Binding to the VAO remains
	}
}

impl RenderPrimitive for SimpleLineGeom {
	fn vao(&self) -> u32 {
		self.vao
	}

	fn draw(&self) {
		draw_arrays(LINES, Self::NUM_VERTICES);
	}
}

impl Geom for SimpleLineGeom {}

trait Mesh : RenderPrimitive {

}

/// Simplest form of a **Mesh**
pub(crate) struct SpriteMesh {
	vao: u32,
	vbo: u32,
	ebo: u32,
}

impl SpriteMesh {
	const INDICES: [u32; 6] = [
		0, 1, 2, // first triangle
		0, 2, 3  // second triangle
	];

	const NUM_ELEMENTS: u32 = 6;

	/// `[x0, y0, x1, y1]`; (0, 0) as bottom-left
	pub(crate) fn new(points: [u32; 4]) -> Self {
		let vao = with_new_vert_arr();
		let [vbo, ebo] = gen_buf_objs();
		let vertices = [
			// positions                    // tex coords
			points[0] as _, points[3] as _, 0.0, 1.0, // top-left
			points[0] as _, points[1] as _, 0.0, 0.0, // bottom-left
			points[2] as _, points[1] as _, 1.0, 0.0, // bottom-right
			points[2] as _, points[3] as _, 1.0, 1.0, // top-right
		];
		buf_obj_with_data(ARRAY_BUFFER, vbo, &vertices, STATIC_DRAW);
		buf_obj_with_data(ELEMENT_ARRAY_BUFFER, ebo, &Self::INDICES, STATIC_DRAW);
		vert_attr_arr(0, 2, NumType::Float, 4, 0); // Position
		vert_attr_arr(1, 2, NumType::Float, 4, 2); // Texture coord
		Self { vao, vbo, ebo } // Note: Binding to the VAO remains
	}
}

impl Mesh for SpriteMesh {}

impl RenderPrimitive for SpriteMesh {
	fn vao(&self) -> u32 {
		self.vao
	}

	fn draw(&self) {
		draw_elements(TRIANGLES, Self::NUM_ELEMENTS);
	}
}

pub(crate) trait PrimModelTransform {
	fn model_matrix(&self, drawing_context: &DrawingContext) -> TMat4<f32>;
}

impl Hash for &dyn PrimModelTransform {
	fn hash<H: Hasher>(&self, state: &mut H) {
		ptr::hash(self, state);
	}
}

impl PartialEq for &dyn PrimModelTransform {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
	}
}

impl Eq for &dyn PrimModelTransform {}

/// Smart-Scaled Mesh depending on the current window size.
/// This transformation works well for a coordinate system with origin in a corner
/// and the object untranslated.
///
/// The scale factor is calculated by: `min(windowWidth / referenceWidth, windowHeight / referenceHeight)`,
/// where the reference size is decided by the dimensions of the window expected.
///
/// The matrix consists of only one scaling matrix.
pub(crate) struct SmartScaling {
	reference_size: (u32, u32),
	param: Option<(ScalingCenteredTranslateParam, (u32, u32))>,
}

pub(crate) enum ScalingCenteredTranslateParam {
	X,
	Y,
	Both,
}

impl SmartScaling {
	pub(crate) fn new(reference_size: (u32, u32), param: Option<(ScalingCenteredTranslateParam, (u32, u32))>) -> Self {
		Self { reference_size, param }
	}
}

impl PrimModelTransform for SmartScaling {
	fn model_matrix(&self, drawing_context: &DrawingContext) -> TMat4<f32> {
		let factor = f32::min(
			drawing_context.window_size.0 as f32 / self.reference_size.0 as f32,
			drawing_context.window_size.1 as f32 / self.reference_size.1 as f32,
		);
		let scaling_vec = vec3(factor, factor, 0.0);
		let scaling_mat = scaling(&scaling_vec);
		match &self.param {
			None => scaling_mat,
			Some(param) => match param.0 {
				ScalingCenteredTranslateParam::X => {
					let vec = vec3(
						(drawing_context.window_size.0 as f32 - param.1.0 as f32 * factor) / 2.0,
						0.0,
						0.0,
					);
					translate(&scaling_mat, &vec)
				},
				ScalingCenteredTranslateParam::Y => {
					let vec = vec3(
						0.0,
						(drawing_context.window_size.1 as f32 - param.1.1 as f32 * factor) / 2.0,
						0.0,
					);
					translate(&scaling_mat, &vec)
				},
				ScalingCenteredTranslateParam::Both => {
					let vec = vec3(
						(drawing_context.window_size.0 as f32 - param.1.0 as f32 * factor) / 2.0,
						(drawing_context.window_size.1 as f32 - param.1.1 as f32 * factor) / 2.0,
						0.0,
					);
					translate(&scaling_mat, &vec)
				},
			}
		}
	}
}

pub(crate) trait PrimColorFilter {
	fn filter_matrix(&self, drawing_context: &DrawingContext) -> TMat4<f32>;
}

impl Hash for &dyn PrimColorFilter {
	fn hash<H: Hasher>(&self, state: &mut H) {
		ptr::hash(self, state);
	}
}

impl PartialEq for &dyn PrimColorFilter {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
	}
}

impl Eq for &dyn PrimColorFilter {}
