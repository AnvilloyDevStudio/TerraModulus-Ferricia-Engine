/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use crate::mui::ogl::GLHandle;
use crate::mui::SdlHandle;
use crate::{FerriciaError, FerriciaResult};
use gl::COLOR_BUFFER_BIT;
use sdl3::video::{SwapInterval, Window, WindowBuildError};
use std::ptr::null;

impl From<WindowBuildError> for FerriciaError {
	fn from(value: WindowBuildError) -> Self {
		value.to_string().into()
	}
}

/// Handles top level functionalities of OpenGL
pub(crate) struct WindowHandle {
	window: Window,
	gl_handle: GLHandle,
}

const MIN_WIDTH: u32 = 800;
const MIN_HEIGHT: u32 = 480;

impl WindowHandle {
	pub(crate) fn new(sdl_handle: &SdlHandle) -> FerriciaResult<Self> {
		let mut window = sdl_handle.video.window("TerraModulus", MIN_WIDTH, MIN_HEIGHT)
			.opengl()
			.hidden()
			.position_centered()
			.resizable()
			.build()?;
		println!("Debug?");
		dbg!("Yes");
		window.set_minimum_size(MIN_WIDTH, MIN_HEIGHT)?;
		let gl_context = window.gl_create_context()?;
		window.gl_make_current(&gl_context)?;
		gl::load_with(|s| sdl_handle.video.gl_get_proc_address(s).map_or(null::<fn()>(), |f| f as *const _) as *const _);
		Ok(Self {
			gl_handle: GLHandle::new(gl_context),
			window,
		})
	}

	fn gl_viewport(&self) {
		let (width, height) = self.window.size();
		self.gl_handle.gl_viewport(width, height);
	}

	pub(crate) fn get_gl_version(&self) -> &str {
		self.gl_handle.full_gl_version()
	}
}
