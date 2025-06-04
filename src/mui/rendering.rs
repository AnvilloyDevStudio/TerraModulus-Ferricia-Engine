/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use crate::mui::ogl::{buf_obj_with_data, compile_shader, gen_buf_objs, new_shader_program, vertex_attrib, with_new_vert_arr};
use crate::mui::window::WindowHandle;
use gl::types::GLenum;
use gl::{AttachShader, BindTexture, CompileShader, CreateProgram, CreateShader, DeleteShader, GenTextures, GenerateMipmap, GetShaderInfoLog, GetShaderiv, LinkProgram, ShaderSource, TexImage2D, TexParameteri, ARRAY_BUFFER, CLAMP_TO_EDGE, COMPILE_STATUS, ELEMENT_ARRAY_BUFFER, FLOAT, FRAGMENT_SHADER, LINEAR, RGB, STATIC_DRAW, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, UNSIGNED_BYTE, VERTEX_SHADER};
use image::imageops::flip_vertical;
use image::ImageReader;
use nalgebra_glm::{identity, ortho, scale, translate, translation, vec3, TMat4};
use semver::Version;
use std::ffi::CString;
use std::fs::read_to_string;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use std::sync::LazyLock;
use crate::FerriciaResult;

static IDENT_MAT_4: LazyLock<TMat4<f32>> = LazyLock::new(identity);

pub(crate) struct CanvasHandle {
	/// Size of Canvas in pixels
	size: (u32, u32),
	ortho_proj_mat: TMat4<f32>,
	/// DO NOT MUTATE
	gl_version: Version,
	/// DO NOT MUTATE
	glsl_version: Version,
}

impl CanvasHandle {
	pub(crate) fn new(window_handle: &WindowHandle) -> Self {
		let size = window_handle.window_size_in_pixels();
		Self {
			ortho_proj_mat: ortho_proj_mat(size),
			size,
			gl_version: window_handle.gl_version().clone(),
			glsl_version: window_handle.glsl_version().clone(),
		}
	}

	pub(crate) fn load_image(&self, path: String) -> u32 {
		let img = ImageReader::open(path)
			.expect("Cannot open image")
			.decode()
			.expect("Cannot decode image")
			.into_rgb8();
		// Image coordinates have a difference direction as OpenGL texture coordinates.
		flip_vertical(&img);
		let mut id = MaybeUninit::uninit();
		unsafe { GenTextures(1, id.as_mut_ptr()); }
		let id = unsafe { id.assume_init() };
		unsafe { BindTexture(TEXTURE_2D, id); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as _); }
		unsafe { TexParameteri(TEXTURE_2D, TEXTURE_MAG_FILTER, LINEAR as _); }
		unsafe {
			TexImage2D(
				TEXTURE_2D,
				0,
				RGB as _,
				img.width() as _,
				img.height() as _,
				0,
				RGB,
				UNSIGNED_BYTE,
				img.as_ptr() as *const _
			);
		}
		unsafe { GenerateMipmap(TEXTURE_2D) }
		id
	}

	pub(crate) fn compile_vector_shader(&self, path: String) -> FerriciaResult<u32> {
		self.compile_shader_from(VERTEX_SHADER, path)
	}

	pub(crate) fn compile_fragment_shader(&self, path: String) -> FerriciaResult<u32> {
		self.compile_shader_from(FRAGMENT_SHADER, path)
	}

	/// In real implementation, a whole preprocessed source is passed instead.
	fn compile_shader_from(&self, kind: GLenum, path: String) -> FerriciaResult<u32> {
		Ok(compile_shader(read_to_string(path).expect("Cannot read the file"), kind)?)
	}

	pub(crate) fn new_shader_program_with(&self, vsh: u32, fsh: u32) -> u32 {
		new_shader_program([vsh, fsh])
	}

	pub(crate) fn render_texture(&self, program: u32, texture: u32) {
		// unsafe { UseProgram(program); }
		// unsafe { BindTexture(TEXTURE_2D, texture); }
		// unsafe { BindVertexArray(vao); }
		// unsafe { DrawElements(TRIANGLES, 6, UNSIGNED_INT, 0 as _) }
	}

	pub(crate) fn refresh_canvas_size(&mut self, width: u32, height: u32) {
		self.size = (width, height);
		self.ortho_proj_mat = ortho_proj_mat(self.size);
	}
}

/// Usage: `unsafe { UniformMatrix4fv(0, 1, FALSE, ortho.as_ptr()) }`
/// 
/// This may be an identity matrix if no model/view matrix is supplied.
fn ortho_proj_mat(size: (u32, u32)) -> TMat4<f32> {
	let (width, height) = size;
	ortho::<f32>(0., width as _, 0., height as _, -1., 1.)
}

struct NormalTexture {
	vao: u32,
	vbo: u32,
	ebo: u32,
}

impl NormalTexture {
	const VERTICES: [f32; 16] = [
		// positions // tex coords
		-1.0,  1.0,  0.0, 1.0, // top-left
		-1.0, -1.0,  0.0, 0.0, // bottom-left
		1.0,  -1.0,  1.0, 0.0, // bottom-right
		1.0,   1.0,  1.0, 1.0, // top-right
	];
	
	const INDICES: [u32; 6] = [
		0, 1, 2, // first triangle
		0, 2, 3  // second triangle
	];
	
	fn new() -> NormalTexture {
		let vao = with_new_vert_arr();
		let [vbo, ebo] = gen_buf_objs();
		buf_obj_with_data(ARRAY_BUFFER, vbo, &Self::VERTICES, STATIC_DRAW);
		buf_obj_with_data(ELEMENT_ARRAY_BUFFER, ebo, &Self::INDICES, STATIC_DRAW);
		vertex_attrib::<f32>(0, 2, FLOAT, 4, 0); // Position
		vertex_attrib::<f32>(1, 2, FLOAT, 4, 2); // Texture coord
		Self { vao, vbo, ebo } // Note: Binding to the VAO remains
	}
}

/// Smart-Scaled Texture depending on the current window size.
///
/// Scaling is calculated by: `min(windowWidth / referenceWidth, windowHeight / referenceHeight)`,
/// where the reference size is decided by the dimensions of the window expected.
///
/// The matrix is done by: `T(c) \* S \* T(-c)`,
/// where `S` is the scaling matrix and `T` is the translating matrix with
/// `c` being the center of rendered texture on the canvas.
struct SmartScaledTexture {
	reference_size: (u32, u32),
}

impl SmartScaledTexture {
	fn new(reference_size: (u32, u32)) -> Self {
		Self { reference_size, }
	}

	fn model_mat(&self, render_center: (u32, u32), window_size: (u32, u32)) -> TMat4<f32> {
		let render_center_vec = vec3::<f32>(render_center.0 as _, render_center.1 as _, 0.);
		let t_1 = translation(&render_center_vec);
		let scaling = f32::min(
			window_size.0 as f32 / self.reference_size.0 as f32,
			window_size.1 as f32 / self.reference_size.1 as f32,
		);
		let scaling_vec = vec3(scaling, scaling, 0.0);
		let s = scale(&t_1, &scaling_vec);
		let render_center_vec_neg = -render_center_vec;
		translate(&s, &render_center_vec_neg)
	}
}
