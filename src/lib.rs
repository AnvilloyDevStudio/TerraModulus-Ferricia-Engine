/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

#![feature(fn_traits)]

#[cfg(feature = "client")]
mod mui;
mod util;

use std::backtrace::Backtrace;
use std::cell::Cell;
#[cfg(feature = "client")]
use crate::mui::rendering::CanvasHandle;
#[cfg(feature = "client")]
use crate::mui::rendering::SpriteMesh;
#[cfg(feature = "client")]
use crate::mui::rendering::{DrawableSet, GeoProgram, SimpleLineGeom, TexProgram};
#[cfg(feature = "client")]
use crate::mui::window::WindowHandle;
#[cfg(feature = "client")]
use crate::mui::MuiEvent;
#[cfg(feature = "client")]
use crate::mui::SdlHandle;
use derive_more::From;
use jni::objects::{JClass, JIntArray, JObject, JString, ReleaseMode};
use jni::sys::{jbyte, jint, jintArray, jlong, jobjectArray, jsize, jstring};
use jni::JNIEnv;
use paste::paste;
use sdl3::pixels::Color;
use std::fmt::Display;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::null;
use crate::mui::rendering::{PrimModelTransform, SmartScaling};

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

macro_rules! resolve_res {
	($res:expr, $t: ty, $env:expr) => {
		match $res {
			Ok(v) => v,
			Err(err) => {
				err.throw_jni($env);
				return jni_null!($t)
			}
		}
	};
}

macro_rules! jni_null {
	($t: ty) => {
		null::<()>() as $t
	};
}

#[inline]
fn jni_from_ptr<'a, T>(ptr: jlong) -> &'a mut T {
	unsafe { &mut *(ptr as *mut T) }
}

fn jni_res_to_ptr<T>(result: FerriciaResult<T>, env: &mut JNIEnv) -> jlong {
	match result {
		Ok(v) => jni_to_ptr(v),
		Err(err) => {
			err.throw_jni(env);
			jni_null!(jlong)
		}
	}
}

fn jni_to_ptr<T>(val: T) -> jlong {
	Box::into_raw(Box::new(val)) as jlong
}

fn jni_drop_with_ptr<T>(ptr: jlong) {
	drop(unsafe { Box::from_raw(ptr as *mut T) })
}

thread_local! {
	static BACKTRACE: Cell<Option<Backtrace>> = const { Cell::new(None) };
}

macro_rules! run_catch {
	($func:block, $t: ty, $env:expr) => {
		match catch_unwind(AssertUnwindSafe(|| $func)) {
			Ok(v) => v,
			Err(err) => {
				let b = BACKTRACE.take().unwrap();
				FerriciaError(format!("{err:?}\n{b:?}")).throw_jni($env);
				jni_null!($t)
			}
		}
	};
	($func:block, $env:expr) => {
		match catch_unwind(AssertUnwindSafe(|| $func)) {
			Ok(v) => v,
			Err(err) => {
				let b = BACKTRACE.take().unwrap();
				FerriciaError(format!("{err:?}\n{b:?}")).throw_jni($env);
			}
		}
	};
}

fn jni_get_string(env: &mut JNIEnv, src: JString) -> String {
	env.get_string(&src).expect("Cannot get Java string").into()
}

// #[allow(non_snake_case)]
// #[unsafe(no_mangle)]
// pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_hello(
// 	mut env: JNIEnv,
// 	class: JClass,
// 	name: JString,
// ) -> jstring {
// 	let input: String =
// 		env.get_string(&name).expect("Couldn't get java string!").into();
// 	let output = env.new_string(format!("Hello, {}!", input))
// 		.expect("Couldn't create java string!");
// 	output.into_raw()
// }

// #[allow(non_snake_case)]
// #[unsafe(no_mangle)]
// pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_clientOnly(
// 	mut env: JNIEnv,
// 	class: JClass,
// ) -> jint {
// 	0
// 	// unsafe { ode_sys::dInitODE2(0); }
// }

macro_rules! jni_ferricia {
	{ $class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) {
				run_catch!($body, &mut $env);
			}
		}
	};
	{ $class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) -> $ret {
				return run_catch!($body, $ret, &mut $env);
			}
		}
	};
	{ client:$class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "client")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) {
				run_catch!($body, &mut $env);
			}
		}
	};
	{ client:$class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "client")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) -> $ret {
				return run_catch!($body, $ret, &mut $env);
			}
		}
	};
	{ server:$class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "server")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) {
				run_catch!($body, &mut $env);
			}
		}
	};
	{ server:$class:ident.$function:ident( mut $env:ident: JNIEnv, $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[allow(clippy::not_unsafe_ptr_arg_deref)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "server")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]
			(mut $env: JNIEnv, $($params)*) -> $ret {
				return run_catch!($body, $ret, &mut $env);
			}
		}
	};
}

jni_ferricia! {
	Core.init(mut env: JNIEnv, class: JClass) {
		// Source: https://stackoverflow.com/a/73711057
		std::panic::set_hook(Box::new(|_| {
			BACKTRACE.set(Some(Backtrace::capture()));
		}));
	}
}

jni_ferricia! {
	client:Mui.initSdlHandle(mut env: JNIEnv, class: JClass) -> jlong {
		jni_res_to_ptr(SdlHandle::new(), &mut env) as jlong
	}
}

jni_ferricia! {
	client:Mui.dropSdlHandle(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_drop_with_ptr::<SdlHandle>(handle);
	}
}

jni_ferricia! {
	client:Mui.initWindowHandle(mut env: JNIEnv, class: JClass, handle: jlong) -> jlong {
		jni_res_to_ptr(WindowHandle::new(jni_from_ptr(handle)), &mut env)
	}
}

jni_ferricia! {
	client:Mui.dropWindowHandle(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_drop_with_ptr::<WindowHandle>(handle);
	}
}

jni_ferricia! {
	client:Mui.getGLVersion(mut env: JNIEnv, class: JClass, handle: jlong) -> jstring {
		env.new_string(jni_from_ptr::<WindowHandle>(handle).full_gl_version())
			.expect("Cannot create Java string")
			.into_raw()
	}
}

jni_ferricia! {
	client:Mui.sdlPoll(mut env: JNIEnv, class: JClass, handle: jlong) -> jobjectArray {
		let v = jni_from_ptr::<SdlHandle>(handle).poll();
		let a = env.new_object_array(v.len() as jsize, "terramodulus/engine/MuiEvent", JObject::null())
			.expect("Cannot create Java array");
		v.into_iter().enumerate().for_each(|(i, e)| {
			let v = match e {
				MuiEvent::DisplayAdded(handle) => {
					let p = vec!(jni_to_ptr(handle).into());
					env.new_object("terramodulus/engine/MuiEvent$DisplayAdded", "(J)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::DisplayRemoved(handle) => {
					let p = vec!(jni_to_ptr(handle).into());
					env.new_object("terramodulus/engine/MuiEvent$DisplayRemoved", "(J)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::DisplayMoved(handle) => {
					let p = vec!(jni_to_ptr(handle).into());
					env.new_object("terramodulus/engine/MuiEvent$DisplayMoved", "(J)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::WindowShown => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowShown";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowHidden => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowHidden";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowExposed => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowExposed";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowMoved(x, y) => {
					let p = vec!(x.into(), y.into());
					env.new_object("terramodulus/engine/MuiEvent$WindowMoved", "(II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::WindowResized(w, h) => {
					let p = vec!(w.into(), h.into());
					env.new_object("terramodulus/engine/MuiEvent$WindowResized", "(II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::WindowPixelSizeChanged(w, h) => {
					let p = vec!(w.into(), h.into());
					env.new_object("terramodulus/engine/MuiEvent$WindowPixelSizeChanged", "(II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::WindowMetalViewResized => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowMetalViewResized";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowMinimized => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowMinimized";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowMaximized => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowMaximized";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowRestored => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowRestored";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowMouseEnter => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowMouseEnter";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowMouseLeave => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowMouseLeave";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowFocusGained => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowFocusGained";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowFocusLost => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowFocusLost";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowCloseRequested => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowCloseRequested";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowIccProfChanged => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowIccProfChanged";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowOccluded => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowOccluded";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowEnterFullscreen => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowEnterFullscreen";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowLeaveFullscreen => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowLeaveFullscreen";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowDestroyed => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowDestroyed";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::WindowHdrStateChanged => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$WindowHdrStateChanged";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::KeyboardKeyDown(id, k) => {
					let p = vec!((id as jint).into(), (k as u32 as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$KeyboardKeyDown", "(II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::KeyboardKeyUp(id, k) => {
					let p = vec!((id as jint).into(), (k as u32 as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$KeyboardKeyUp", "(II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::TextEditing(t, s, l) => {
					let ss = env.new_string(t).expect("Cannot create Java string");
					let p = vec!((&ss).into(), s.into(), l.into());
					env.new_object("terramodulus/engine/MuiEvent$TextEditing", "(Ljava/lang/String;II)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::TextInput(t) => {
					let ss = env.new_string(t).expect("Cannot create Java string");
					let p = vec!((&ss).into());
					env.new_object("terramodulus/engine/MuiEvent$TextInput", "(Ljava/lang/String;)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::KeymapChanged => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$KeymapChanged";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::KeyboardAdded => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$KeyboardAdded";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::KeyboardRemoved => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$KeyboardRemoved";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::TextEditingCandidates => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$TextEditingCandidates";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::MouseMotion(id, x, y) => {
					let p = vec!((id as jint).into(), x.into(), y.into());
					env.new_object("terramodulus/engine/MuiEvent$MouseMotion", "(IFF)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::MouseButtonDown(id, k) => {
					let p = vec!((id as jint).into(), (k as u8 as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$MouseButtonDown", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::MouseButtonUp(id, k) => {
					let p = vec!((id as jint).into(), (k as u8 as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$MouseButtonUp", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::MouseWheel(id, x, y) => {
					let p = vec!((id as jint).into(), x.into(), y.into());
					env.new_object("terramodulus/engine/MuiEvent$MouseWheel", "(IFF)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::MouseAdded => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$MouseAdded";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::MouseRemoved => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$MouseRemoved";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::JoystickAxisMotion(id, a , v) => {
					let p = vec!((id as jint).into(), (a as jbyte).into(), v.into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickAxisMotion", "(IBS)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickBallMotion => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$JoystickBallMotion";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::JoystickHatMotion(id, h , s) => {
					let p = vec!((id as jint).into(), (h as jbyte).into(), (s as u8 as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickHatMotion", "(IBB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickButtonDown(id, b) => {
					let p = vec!((id as jint).into(), (b as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickButtonDown", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickButtonUp(id, b) => {
					let p = vec!((id as jint).into(), (b as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickButtonUp", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickAdded(id) => {
					let p = vec!((id as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickAdded", "(I)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickRemoved(id) => {
					let p = vec!((id as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$JoystickRemoved", "(I)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::JoystickBatteryUpdated => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$JoystickBatteryUpdated";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::GamepadAxisMotion(id, a , v) => {
					let p = vec!((id as jint).into(), (a as u8 as jbyte).into(), v.into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadAxisMotion", "(IBS)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadButtonDown(id, b) => {
					let p = vec!((id as jint).into(), (b as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadButtonDown", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadButtonUp(id, b) => {
					let p = vec!((id as jint).into(), (b as jbyte).into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadButtonUp", "(IB)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadAdded(id) => {
					let p = vec!((id as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadAdded", "(I)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadRemoved(id) => {
					let p = vec!((id as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadRemoved", "(I)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadRemapped(id) => {
					let p = vec!((id as jint).into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadRemapped", "(I)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadTouchpadDown(id, t, f, x, y, p) => {
					let p = vec!((id as jint).into(), t.into(), f.into(), x.into(), y.into(), p.into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadTouchpadDown", "(IIIFFF)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadTouchpadMotion(id, t, f, x, y, p) => {
					let p = vec!((id as jint).into(), t.into(), f.into(), x.into(), y.into(), p.into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadTouchpadMotion", "(IIIFFF)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadTouchpadUp(id, t, f, x, y, p) => {
					let p = vec!((id as jint).into(), t.into(), f.into(), x.into(), y.into(), p.into());
					env.new_object("terramodulus/engine/MuiEvent$GamepadTouchpadUp", "(IIIFFF)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::GamepadSteamHandleUpdated => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$GamepadSteamHandleUpdated";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::DropFile(f) => {
					let ss = env.new_string(f).expect("Cannot create Java string");
					let p = vec!((&ss).into());
					env.new_object("terramodulus/engine/MuiEvent$DropFile", "(Ljava/lang/String;)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::DropText(t) => {
					let ss = env.new_string(t).expect("Cannot create Java string");
					let p = vec!((&ss).into());
					env.new_object("terramodulus/engine/MuiEvent$DropText", "(Ljava/lang/String;)V", p.as_slice())
						.expect("Cannot create Java object")
				}
				MuiEvent::DropBegin => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$DropBegin";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::DropComplete => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$DropComplete";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::DropPosition => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$DropPosition";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::RenderTargetsReset => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$RenderTargetsReset";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::RenderDeviceReset => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$RenderDeviceReset";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
				MuiEvent::RenderDeviceLost => {
					const CLASS: &str = "terramodulus/engine/MuiEvent$RenderDeviceLost";
					env.get_static_field(CLASS, "INSTANCE", format!("L{CLASS};"))
						.expect("Cannot get static field")
						.l()
						.expect("JObject is expected")
				}
			};
			env.set_object_array_element(&a, i as jsize, v).expect("Cannot set Java object array");
		});
		a.into_raw()
	}
}

jni_ferricia! {
	client:Mui.resizeGLViewport(mut env: JNIEnv, class: JClass, handle: jlong, canvas_handle: jlong) {
		jni_from_ptr::<WindowHandle>(handle).gl_resize_viewport(jni_from_ptr::<CanvasHandle>(canvas_handle));
	}
}

jni_ferricia! {
	client:Mui.showWindow(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_from_ptr::<WindowHandle>(handle).show_window()
	}
}

jni_ferricia! {
	client:Mui.swapWindow(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_from_ptr::<WindowHandle>(handle).swap_window()
	}
}

jni_ferricia! {
	client:Mui.initCanvasHandle(mut env: JNIEnv, class: JClass, handle: jlong) -> jlong {
		jni_to_ptr(CanvasHandle::new(jni_from_ptr::<WindowHandle>(handle)))
	}
}

jni_ferricia! {
	client:Mui.dropCanvasHandle(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_drop_with_ptr::<CanvasHandle>(handle);
	}
}

jni_ferricia! {
	client:Mui.loadImageToCanvas(mut env: JNIEnv, class: JClass, handle: jlong, path: JString) -> jint {
		jni_from_ptr::<CanvasHandle>(handle).load_image(env.get_string(&path)
			.expect("Cannot get Java string").into()) as jint
	}
}

jni_ferricia! {
	client:Mui.geoShaders(mut env: JNIEnv, class: JClass, vsh: JString, fsh: JString) -> jlong {
		jni_res_to_ptr(GeoProgram::new(jni_get_string(&mut env, vsh), jni_get_string(&mut env, fsh)), &mut env)
	}
}

jni_ferricia! {
	client:Mui.texShaders(mut env: JNIEnv, class: JClass, vsh: JString, fsh: JString) -> jlong {
		jni_res_to_ptr(TexProgram::new(jni_get_string(&mut env, vsh), jni_get_string(&mut env, fsh)), &mut env)
	}
}

jni_ferricia! {
	client:Mui.renderTexture(mut env: JNIEnv, class: JClass, handle: jlong, shader: jint, texture: jint) {
		// jni_from_ptr::<CanvasHandle>(handle).render_texture(SpriteMesh::new(), shader as _, texture as _)
	}
}

jni_ferricia! {
	client:Mui.newSimpleLineGeom(mut env: JNIEnv, class: JClass, data: jintArray) -> jlong {
		let data = unsafe { JIntArray::from_raw(data) };
		let arr = unsafe {
			env.get_array_elements(&data, ReleaseMode::NoCopyBack)
				.expect("Cannot get Java array elements")
		};
		let arr = arr.get(0..8).expect("Cannot get Java array elements");
		jni_to_ptr(DrawableSet::new(SimpleLineGeom::new(
			[(arr[0] as f32, arr[1] as f32), (arr[2] as f32, arr[3] as f32)],
			Color::RGBA(arr[4] as u8, arr[5] as u8, arr[6] as u8, arr[7] as u8),
		)))
	}
}

jni_ferricia! {
	client:Mui.newSpriteMesh(mut env: JNIEnv, class: JClass, data: jintArray) -> jlong {
		let data = unsafe { JIntArray::from_raw(data) };
		let arr = unsafe {
			env.get_array_elements(&data, ReleaseMode::NoCopyBack)
				.expect("Cannot get Java array elements")
		};
		let arr = arr.get(0..4).expect("Cannot get Java array elements");
		jni_to_ptr(DrawableSet::new(SpriteMesh::new([arr[0] as _, arr[1] as _, arr[2] as _, arr[3] as _])))
	}
}

jni_ferricia! {
	client:Mui.modelSmartScaling(mut env: JNIEnv, class: JClass, data: jintArray) -> jlong {
		let data = unsafe { JIntArray::from_raw(data) };
		let arr = unsafe {
			env.get_array_elements(&data, ReleaseMode::NoCopyBack)
				.expect("Cannot get Java array elements")
		};
		let arr = arr.get(0..2).expect("Cannot get Java array elements");
		jni_to_ptr(SmartScaling::new((arr[0] as _, arr[1] as _)))
	}
}

jni_ferricia! {
	client:Mui.addSmartScaling(mut env: JNIEnv, class: JClass, set_handle: jlong, model_handle: jlong) {
		jni_from_ptr::<DrawableSet>(set_handle).add_model_transform(jni_from_ptr::<SmartScaling>(model_handle))
	}
}

jni_ferricia! {
	client:Mui.removeSmartScaling(mut env: JNIEnv, class: JClass, set_handle: jlong, model_handle: jlong) {
		jni_from_ptr::<DrawableSet>(set_handle).remove_model_transform(jni_from_ptr::<SmartScaling>(model_handle))
	}
}

jni_ferricia! {
	client:Mui.drawGuiGeo(
		mut env: JNIEnv,
		class: JClass,
		canvas_handle: jlong,
		drawable_handle: jlong,
		program_handle: jlong,
	) {
		jni_from_ptr::<CanvasHandle>(canvas_handle)
			.draw_gui(jni_from_ptr::<DrawableSet>(drawable_handle), jni_from_ptr::<GeoProgram>(program_handle), None)
	}
}

jni_ferricia! {
	client:Mui.drawGuiTex(
		mut env: JNIEnv,
		class: JClass,
		canvas_handle: jlong,
		drawable_handle: jlong,
		program_handle: jlong,
		texture_handle: jint,
	) {
		jni_from_ptr::<CanvasHandle>(canvas_handle).draw_gui(
			jni_from_ptr::<DrawableSet>(drawable_handle),
			jni_from_ptr::<TexProgram>(program_handle),
			Some(texture_handle as _),
		)
	}
}
