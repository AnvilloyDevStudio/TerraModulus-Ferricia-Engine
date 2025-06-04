/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

//! High level OpenGL
//!
//! Current target minimal OpenGL version to support is **2.0**, with advanced features supported
//! in newer versions used only in newer versions detected.
//!
//! Although OpenGL 2.0 core does support VAO, VAO is still used due to its simplicity and performance
//! improvement, and the complexity of cross-version maintenance. If the version is lower than 3.0,
//! the extension of `GL_ARB_vertex_array_object` is required.
//!
//! To not waste GL callsites, clearing bindings is generally not used, so when mutations are
//! done, one should ensure that the target desired object has already been bound.
//! All renderings should use VAOs regardless to keep uniform patterns across the Engine.
//! Please keep in mind that binding existing VAOs may replace the states of EBO binding and
//! vertex attributes when they were set, but not for VBOs, so this must be taken carefully.
//! During rendering using VAOs, all the used objects must be bound in the VAO as desired.

use std::cmp::Ordering;
use getset::Getters;
use gl::types::{GLenum, GLubyte, GLuint};
use gl::{AttachShader, BindBuffer, BindVertexArray, BufferData, CompileShader, CreateProgram, CreateShader, DeleteShader, EnableVertexAttribArray, GenBuffers, GenVertexArrays, GetIntegerv, GetShaderInfoLog, GetShaderiv, GetString, GetStringi, LinkProgram, ShaderSource, VertexAttribPointer, Viewport, ARRAY_BUFFER, COMPILE_STATUS, EXTENSIONS, FALSE, FLOAT, NUM_EXTENSIONS, RENDERER, SHADING_LANGUAGE_VERSION, VENDOR, VERSION};
use num_traits::{Bounded, Num};
use regex::Regex;
use sdl3::video::GLContext;
use semver::Version;
use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::fs::read_to_string;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use std::sync::LazyLock;

const VER_2_0: Version = Version::new(2, 0, 0);
const VER_3_0: Version = Version::new(3, 0, 0);

#[derive(Getters)]
pub(super) struct GLHandle {
	gl_context: GLContext,
	#[get = "pub"]
	vendor: String,
	#[get = "pub"]
	renderer: String,
	#[get = "pub"]
	full_gl_version: String,
	#[get = "pub"]
	gl_version: Version,
	#[get = "pub"]
	full_glsl_version: String,
	#[get = "pub"]
	glsl_version: Version,
	extensions: HashSet<String>,
}

/// Supposed to be **immutable**.
impl GLHandle {
	/// Make sure context is current and function pointer is handled before this.
	pub(crate) fn new(gl_context: GLContext) -> Result<Self, String> {
		let full_gl_version = get_gl_version();
		let full_glsl_version = get_glsl_version();
		let instance = Self {
			gl_context,
			vendor: get_vendor(),
			renderer: get_renderer(),
			gl_version: parse_version(&full_gl_version),
			full_gl_version,
			glsl_version: parse_version(&full_glsl_version),
			full_glsl_version,
			extensions: get_extensions()
		};
		instance.check_requirements()?;
		Ok(instance)
	}

	/// Since mobile platforms are not supported, OpenGL ES and OES extensions are not relevant.
	fn check_requirements(&self) -> Result<(), String> {
		if self.gl_version.cmp(&VER_2_0) == Ordering::Less { // < 2.0
			return Err(format!("GL {} not supported", self.gl_version));
		} else if self.gl_version.cmp(&VER_3_0) == Ordering::Less { // < 3.0
			if !self.extensions.contains("GL_ARB_vertex_array_object") {
				return Err(format!("GL_ARB_vertex_array_object not found with GL {}", self.gl_version));
			}
		}

		Ok(())
	}

	pub(super) fn gl_resize_viewport(&self, width: u32, height: u32) {
		unsafe { Viewport(0, 0, width as i32, height as i32) }
	}
}

fn get_vendor() -> String {
	unsafe { str_from_gl(GetString(VENDOR)).to_string() }
}

fn get_renderer() -> String {
	unsafe { str_from_gl(GetString(RENDERER)).to_string() }
}

fn get_gl_version() -> String {
	unsafe { str_from_gl(GetString(VERSION)).to_string() }
}

fn get_glsl_version() -> String {
	unsafe { str_from_gl(GetString(SHADING_LANGUAGE_VERSION)).to_string() }
}

fn get_extensions() -> HashSet<String> {
	let mut data = MaybeUninit::uninit();
	unsafe { GetIntegerv(NUM_EXTENSIONS, data.as_mut_ptr()); }
	let num = unsafe { data.assume_init() } as u32;
	let mut data = HashSet::with_capacity(num as usize);
	for i in 0..num {
		data.insert(str_from_gl(unsafe { GetStringi(EXTENSIONS, i as GLuint) }).to_string());
	}
	data
}

static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\.(\d+)").expect("invalid regex"));

/// Only parses the first two parts (major, minor) of the version string.
fn parse_version(version_str: &str) -> Version {
	let caps = VERSION_REGEX.captures(version_str).expect("invalid version string");
	Version::new(caps[1].parse().unwrap(), caps[2].parse().unwrap(), 0)
}

fn str_from_gl(string: *const GLubyte) -> &'static str {
	unsafe { CStr::from_ptr(string as *const _).to_str().expect("should be valid utf8") }
}

/// Generate a single Buffer Object.
pub(super) fn gen_buf_obj() -> u32 {
	let mut bo = MaybeUninit::uninit();
	unsafe { GenBuffers(1, bo.as_mut_ptr()); }
	unsafe { bo.assume_init() }
}

/// Generate multiple Buffer Objects at once for optimization.
pub(super) fn gen_buf_objs<const N: usize>() -> [u32; N] {
	let mut bos = MaybeUninit::uninit();
	unsafe { GenBuffers(N as _, bos.as_mut_ptr() as *mut _); }
	unsafe { bos.assume_init() }
}

/// Generate a single Vertex Array Object.
pub(super) fn gen_vert_arr_obj() -> u32 {
	let mut vao = MaybeUninit::uninit();
	unsafe { GenVertexArrays(1, vao.as_mut_ptr()); }
	unsafe { vao.assume_init() }
}

/// Generate multiple Vertex Array Objects at once for optimization.
pub(super) fn gen_vert_arr_objs<const N: usize>() -> [u32; N] {
	let mut vaos = MaybeUninit::uninit();
	unsafe { GenVertexArrays(N as _, vaos.as_mut_ptr() as *mut _); }
	unsafe { vaos.assume_init() }
}

pub(super) trait Number : Num + Bounded {}

impl<T: Num + Bounded> Number for T {}

pub(super) fn buf_obj_with_data<T: Number>(target: GLenum, buffer: u32, data: &[T], usage: GLenum) {
	unsafe { BindBuffer(ARRAY_BUFFER, buffer); }
	unsafe { BufferData(target, size_of_val(data) as _, data.as_ptr() as _, usage); }
}

/// Defines an array of Vertex Attribute. Normalized is not applied.
pub(super) fn vertex_attrib<T: Number>(i: u32, vec_size: usize, kind: GLenum, stride_len: usize, offset_len: usize) {
	unsafe { EnableVertexAttribArray(i); }
	unsafe {
		VertexAttribPointer(
			i,
			vec_size as _,
			kind,
			FALSE,
			(stride_len * size_of::<T>()) as _,
			(offset_len * size_of::<T>()) as _,
		);
	}
}

pub(super) fn with_new_vert_arr() -> u32 {
	let vao = gen_vert_arr_obj();
	unsafe { BindVertexArray(vao); }
	vao
}

/// `src` should not contain any `\0` char.
pub(super) fn compile_shader(src: String, kind: GLenum) -> Result<u32, String> {
	let shader = unsafe { CreateShader(kind) };
	let src = CString::new(src).expect("Cannot create CString").into_raw() as *const _;
	unsafe { ShaderSource(shader, 1, &src, null()); }
	unsafe { CompileShader(shader); }
	let mut status = MaybeUninit::uninit();
	unsafe { GetShaderiv(shader, COMPILE_STATUS, status.as_mut_ptr()); }
	if unsafe { status.assume_init() } == FALSE as _ {
		let out = CString::default().into_raw();
		unsafe { GetShaderInfoLog(shader, COMPILE_STATUS as _, null_mut(), out); }
		let out = unsafe { CString::from_raw(out) };
		return Err(out.to_str().expect("Invalid UTF-8 CString").to_string());
	}
	Ok(shader)
}

pub(super) fn new_shader_program<const N: usize>(shaders: [u32; N]) -> u32 {
	let program = unsafe { CreateProgram() };
	shaders.iter().for_each(|s| unsafe { AttachShader(program, *s) });
	unsafe { LinkProgram(program); }
	shaders.into_iter().for_each(|s| unsafe { DeleteShader(s) });
	program
}
