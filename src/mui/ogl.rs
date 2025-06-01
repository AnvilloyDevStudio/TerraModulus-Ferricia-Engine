/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

//! High level OpenGL
//!
//! Current target minimal OpenGL version to support is **2.0**, with advanced features supported
//! in newer versions used only in newer versions detected.

use std::collections::HashSet;
use gl::types::{GLubyte, GLuint};
use gl::{Clear, ClearColor, GetIntegerv, GetString, GetStringi, Viewport, COLOR_BUFFER_BIT, EXTENSIONS, NUM_EXTENSIONS, RENDERER, SHADING_LANGUAGE_VERSION, VENDOR, VERSION};
use regex::Regex;
use sdl3::video::GLContext;
use semver::Version;
use std::ffi::CStr;
use std::ptr::null_mut;
use std::sync::LazyLock;
use getset::Getters;

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

impl GLHandle {
	/// Make sure context is current and function pointer is handled before this.
	pub(crate) fn new(gl_context: GLContext) -> Self {
		let full_gl_version = get_gl_version();
		let full_glsl_version = get_glsl_version();
		Self {
			gl_context,
			vendor: get_vendor(),
			renderer: get_renderer(),
			gl_version: parse_version(&full_gl_version),
			full_gl_version,
			glsl_version: parse_version(&full_glsl_version),
			full_glsl_version,
			extensions: get_extensions()
		}
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
	let data = null_mut();
	unsafe { GetIntegerv(NUM_EXTENSIONS, data); }
	let num = unsafe { *data } as u32;
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
