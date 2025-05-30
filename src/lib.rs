/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

#[cfg(feature = "client")]
mod mui;

#[cfg(feature = "client")]
use crate::mui::init_sdl_handle;
#[cfg(feature = "client")]
use crate::mui::window::init_window_handle;
use derive_more::From;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jstring};
use jni::JNIEnv;
use std::fmt::Display;
use std::ptr::null;
#[cfg(feature = "client")]
use crate::mui::SdlHandle;
use crate::mui::window::get_gl_version;
#[cfg(feature = "client")]
use crate::mui::window::WindowHandle;

#[derive(From)]
struct FerriciaError(String);

impl FerriciaError {
	fn throw_jni(self, env: &mut JNIEnv) {
		handle_jni_error(env.throw_new("terramodulus/util/exception/FerriciaEngineFault", self.0), env);
	}
}

#[allow(unused_variables)]
fn handle_jni_error<E: Display>(result: Result<(), E>, env: &mut JNIEnv) {
	match result {
		Ok(_) => {},
		Err(err) => {
			#[cfg(debug_assertions)]
			panic!("{}", err);
			#[cfg(not(debug_assertions))]
			FerriciaError(err.to_string()).throw_jni(env);
		}
	}
}

type FerriciaResult<T> = Result<T, FerriciaError>;

#[inline]
fn jni_null() -> jlong {
	null::<()>() as jlong
}

#[inline]
fn jni_from_ptr<'a, T>(ptr: jlong) -> &'a mut T {
	unsafe { &mut *(ptr as *mut T) }
}

fn jni_to_ptr<T>(env: &mut JNIEnv, result: FerriciaResult<T>) -> jlong {
	match result {
		Ok(v) => Box::into_raw(Box::new(v)) as jlong,
		Err(err) => {
			err.throw_jni(env);
			jni_null()
		}
	}
}

fn jni_drop_with_ptr<T>(ptr: jlong) {
	drop(unsafe { Box::from_raw(ptr as *mut T) })
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_hello(
	mut env: JNIEnv,
	class: JClass,
	name: JString,
) -> jstring {
	let input: String =
		env.get_string(&name).expect("Couldn't get java string!").into();
	let output = env.new_string(format!("Hello, {}!", input))
		.expect("Couldn't create java string!");
	output.into_raw()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_clientOnly(
	mut env: JNIEnv,
	class: JClass,
) -> jint {
	0
	// unsafe { ode_sys::dInitODE2(0); }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[cfg(feature = "client")]
pub extern "system" fn Java_terramodulus_engine_ferricia_UI_initSdlHandle(
	mut env: JNIEnv,
	_class: JClass,
) -> jlong {
	jni_to_ptr(&mut env, init_sdl_handle())
}

#[allow(unused_mut)]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[cfg(feature = "client")]
pub extern "system" fn Java_terramodulus_engine_ferricia_UI_dropSdlHandle(
	mut _env: JNIEnv,
	_class: JClass,
	handle: jlong,
) {
	jni_drop_with_ptr::<SdlHandle>(handle);
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[cfg(feature = "client")]
pub extern "system" fn Java_terramodulus_engine_ferricia_UI_initWindowHandle(
	mut env: JNIEnv,
	_class: JClass,
	handle: jlong,
) -> jlong {
	jni_to_ptr(&mut env, init_window_handle(jni_from_ptr(handle)))
}

#[allow(unused_mut)]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[cfg(feature = "client")]
pub extern "system" fn Java_terramodulus_engine_ferricia_UI_dropWindowHandle(
	mut _env: JNIEnv,
	_class: JClass,
	handle: jlong,
) {
	jni_drop_with_ptr::<WindowHandle>(handle);
}

#[allow(unused_mut)]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
#[cfg(feature = "client")]
pub extern "system" fn Java_terramodulus_engine_ferricia_UI_getGLVersion(
	mut env: JNIEnv,
	_class: JClass,
) -> jstring {
	env.new_string(get_gl_version()).expect("Couldn't create java string!").into_raw()
}
