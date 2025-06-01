/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

#[cfg(feature = "client")]
mod mui;

#[cfg(feature = "client")]
use crate::mui::window::WindowHandle;
#[cfg(feature = "client")]
use crate::mui::SdlHandle;
use derive_more::From;
use jni::objects::{JClass, JObject, JValueOwned};
use jni::sys::{jarray, jbyte, jint, jlong, jobject, jobjectArray, jsize, jstring};
use jni::JNIEnv;
use std::fmt::Display;
use std::ptr::null;
use paste::paste;
use crate::mui::MuiEvent;

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

fn jni_res_to_ptr<T>(env: &mut JNIEnv, result: FerriciaResult<T>) -> jlong {
	match result {
		Ok(v) => jni_to_ptr(v) as jlong,
		Err(err) => {
			err.throw_jni(env);
			jni_null()
		}
	}
}

fn jni_to_ptr<T>(val: T) -> jlong {
	Box::into_raw(Box::new(val)) as jlong
}

fn jni_drop_with_ptr<T>(ptr: jlong) {
	drop(unsafe { Box::from_raw(ptr as *mut T) })
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
	{ $class:ident.$function:ident( $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]( $($params)* ) $body
		}
	};
	{ $class:ident.$function:ident( $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]( $($params)* )->$ret $body
		}
	};
	{ client:$class:ident.$function:ident( $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "client")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]($($params)*) $body
		}
	};
	{ client:$class:ident.$function:ident( $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "client")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]($($params)*)->$ret $body
		}
	};
	{ server:$class:ident.$function:ident( $($params:tt)* ) $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "server")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]($($params)*) $body
		}
	};
	{ server:$class:ident.$function:ident( $($params:tt)* ) -> $ret:ty $body:block } => {
		paste! {
			#[allow(unused_mut)]
			#[allow(unused_variables)]
			#[allow(non_snake_case)]
			#[unsafe(no_mangle)]
			#[cfg(feature = "server")]
			pub extern "system" fn [<Java_terramodulus_engine_ferricia_ $class _ $function>]($($params)*)->$ret $body
		}
	};
}

jni_ferricia! {
	client:Mui.initSdlHandle(mut env: JNIEnv, class: JClass) -> jlong {
		jni_res_to_ptr(&mut env, SdlHandle::new())
	}
}

jni_ferricia! {
	client:Mui.dropSdlHandle(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_drop_with_ptr::<SdlHandle>(handle);
	}
}

jni_ferricia! {
	client:Mui.initWindowHandle(mut env: JNIEnv, class: JClass, handle: jlong) -> jlong {
		jni_res_to_ptr(&mut env, WindowHandle::new(jni_from_ptr(handle)))
	}
}

jni_ferricia! {
	client:Mui.dropWindowHandle(mut env: JNIEnv, class: JClass, handle: jlong) {
		jni_drop_with_ptr::<WindowHandle>(handle);
	}
}

jni_ferricia! {
	client:Mui.getGLVersion(mut env: JNIEnv, class: JClass, handle: jlong) -> jstring {
		env.new_string(jni_from_ptr::<WindowHandle>(handle).gl_version())
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
