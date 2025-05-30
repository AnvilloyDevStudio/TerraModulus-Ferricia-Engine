/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use crate::mui::ogl::GLHandle;
use crate::mui::SdlHandle;
use crate::{FerriciaError, FerriciaResult};
use sdl3::video::{Window, WindowBuildError};
use std::ffi::CStr;
use std::ptr::null;
use gl::COLOR_BUFFER_BIT;

impl From<WindowBuildError> for FerriciaError {
	fn from(value: WindowBuildError) -> Self {
		value.to_string().into()
	}
}

pub(crate) struct WindowHandle {
	window: Window,
	gl_handle: GLHandle,
}

const MIN_WIDTH: u32 = 800;
const MIN_HEIGHT: u32 = 480;

pub(crate) fn init_window_handle(sdl_handle: &SdlHandle) -> FerriciaResult<WindowHandle> {
	let mut window = sdl_handle.video.window("TerraModulus", MIN_WIDTH, MIN_HEIGHT)
		.opengl()
		// .hidden()
		.position_centered()
		.resizable()
		.build()?;
	window.set_minimum_size(MIN_WIDTH, MIN_HEIGHT)?;
	let gl_context = window.gl_create_context()?;
	window.gl_make_current(&gl_context)?;
	gl::load_with(|s| sdl_handle.video.gl_get_proc_address(s).map_or(null::<fn()>(), |f| f as *const _) as *const _);
	unsafe { gl::Viewport(0, 0, MIN_WIDTH as i32, MIN_HEIGHT as i32); }
	unsafe { gl::ClearColor(0.3, 0.3, 0.5, 1.0) }
	unsafe { gl::Clear(COLOR_BUFFER_BIT) }
	window.gl_swap_window();
	Ok(WindowHandle {
		gl_handle: GLHandle::new(gl_context),
		window,
	})
}

pub(crate) fn get_gl_version() -> &'static str {
	unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_str().unwrap() }
}
