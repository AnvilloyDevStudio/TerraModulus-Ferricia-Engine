/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use std::ffi::CString;
use std::fs::read_to_string;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use gl::{AttachShader, BindBuffer, BindTexture, BindVertexArray, BufferData, CompileShader, CreateProgram, CreateShader, DeleteShader, DrawElements, EnableVertexAttribArray, GenBuffers, GenTextures, GenVertexArrays, GenerateMipmap, GetShaderInfoLog, GetShaderiv, LinkProgram, ShaderSource, TexImage2D, TexParameteri, UseProgram, VertexAttribPointer, ARRAY_BUFFER, CLAMP_TO_EDGE, COMPILE_STATUS, ELEMENT_ARRAY_BUFFER, FALSE, FLOAT, FRAGMENT_SHADER, LINEAR, RGB, STATIC_DRAW, TEXTURE_2D, TEXTURE_MAG_FILTER, TEXTURE_MIN_FILTER, TEXTURE_WRAP_S, TEXTURE_WRAP_T, TRIANGLES, UNSIGNED_BYTE, UNSIGNED_INT, VERTEX_SHADER};
use gl::types::GLenum;
use image::ImageReader;

pub(crate) struct CanvasHandle {

}

impl CanvasHandle {
	pub(crate) fn new() -> Self {
		Self { }
	}

	pub(crate) fn load_image(&self, path: String) -> u32 {
		let img = ImageReader::open(path)
			.expect("Cannot open image")
			.decode()
			.expect("Cannot decode image")
			.into_rgb8();
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

	pub(crate) fn compile_vector_shader(&self, path: String) -> u32 {
		self.compile_shader(VERTEX_SHADER, path)
	}

	pub(crate) fn compile_fragment_shader(&self, path: String) -> u32 {
		self.compile_shader(FRAGMENT_SHADER, path)
	}

	fn compile_shader(&self, kind: GLenum, path: String) -> u32 {
		let shader = unsafe { CreateShader(kind) };
		let src = CString::new(read_to_string(path).expect("Cannot read the file"))
			.expect("Cannot create CString");
		let src_ptr = src.as_ptr();
		unsafe { ShaderSource(shader, 1, &src_ptr, null()); }
		unsafe { CompileShader(shader); }
		let mut status = MaybeUninit::uninit();
		unsafe { GetShaderiv(shader, COMPILE_STATUS, status.as_mut_ptr()); }
		let status = unsafe { status.assume_init() };
		if status == 0 {
			let out = CString::default().into_raw();
			unsafe { GetShaderInfoLog(shader, COMPILE_STATUS as _, null_mut(), out); }
			let out = unsafe { CString::from_raw(out) };
			panic!("{:?}", out) // TODO error
		}
		shader
	}

	pub(crate) fn new_shader_program(&self, vsh: u32, fsh: u32) -> u32 {
		let program = unsafe { CreateProgram() };
		unsafe { AttachShader(program, vsh); }
		unsafe { AttachShader(program, fsh); }
		unsafe { LinkProgram(program); }
		unsafe { DeleteShader(vsh); }
		unsafe { DeleteShader(fsh); }
		program
	}

	pub(crate) fn render_texture(&self, program: u32, texture: u32) {
		let vertices: [f32; 20] = [
			// positions       // tex coords
			-1.0,  1.0, 0.0,  0.0, 1.0, // top-left
			-1.0, -1.0, 0.0,  0.0, 0.0, // bottom-left
			1.0, -1.0, 0.0,  1.0, 0.0, // bottom-right
			1.0,  1.0, 0.0,  1.0, 1.0  // top-right
		];
		let indices = [
			0, 1, 2, // first triangle
			0, 2, 3  // second triangle
		];
		let mut vao = MaybeUninit::uninit();
		let mut vbo = MaybeUninit::uninit();
		let mut ebo = MaybeUninit::uninit();
		unsafe { GenVertexArrays(1, vao.as_mut_ptr()); }
		unsafe { GenBuffers(1, vbo.as_mut_ptr()); }
		unsafe { GenBuffers(1, ebo.as_mut_ptr()) }
		let vao = unsafe { vao.assume_init() };
		let vbo = unsafe { vbo.assume_init() };
		let ebo = unsafe { ebo.assume_init() };
		unsafe { BindVertexArray(vao); }
		unsafe { BindBuffer(ARRAY_BUFFER, vbo); }
		unsafe { BufferData(ARRAY_BUFFER, size_of_val(&vertices) as _, vertices.as_ptr() as _, STATIC_DRAW); }
		unsafe { BindBuffer(ELEMENT_ARRAY_BUFFER, ebo); }
		unsafe { BufferData(ELEMENT_ARRAY_BUFFER, size_of_val(&indices) as _, indices.as_ptr() as _, STATIC_DRAW); }

		// Position attribute
		unsafe { VertexAttribPointer(0, 3, FLOAT, FALSE, (5 * size_of::<f32>()) as _, 0 as _); }
		unsafe { EnableVertexAttribArray(0); }
		// Texture coord attribute
		unsafe { VertexAttribPointer(1, 2, FLOAT, FALSE, (5 * size_of::<f32>()) as _, (3 * size_of::<f32>()) as _); }
		unsafe { EnableVertexAttribArray(1); }

		unsafe { UseProgram(program); }
		unsafe { BindTexture(TEXTURE_2D, texture); }
		unsafe { BindVertexArray(vao); }
		unsafe { DrawElements(TRIANGLES, 6, UNSIGNED_INT, null()) }
	}
}
