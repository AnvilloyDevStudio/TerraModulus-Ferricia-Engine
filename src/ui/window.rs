/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use sdl3::video::{GLContext, Window, WindowBuildError};
use crate::{FerriciaError, FerriciaResult};
use crate::ui::SdlHandle;

impl From<WindowBuildError> for FerriciaError {
	fn from(value: WindowBuildError) -> Self {
		value.to_string().into()
	}
}

pub(crate) struct WindowHandle {
	window: Window,
	gl_context: GLContext
}

const MIN_WIDTH: u32 = 800;
const MIN_HEIGHT: u32 = 480;

pub(crate) fn init_window_handle(sdl_handle: &SdlHandle) -> FerriciaResult<WindowHandle> {
	let mut window = sdl_handle.video.window("TerraModulus", MIN_WIDTH, MIN_HEIGHT)
		.opengl()
		.hidden()
		.position_centered()
		.resizable()
		.build()?;
	window.set_minimum_size(MIN_WIDTH, MIN_HEIGHT)?;
	Ok(WindowHandle {
		gl_context: window.gl_create_context()?,
		window,
	})
}
