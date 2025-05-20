/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

use std::time::Duration;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_hello(mut env: JNIEnv, class: JClass, name: JString) -> jstring {
	let input: String =
		env.get_string(&name).expect("Couldn't get java string!").into();
	let output = env.new_string(format!("Hello, {}!", input))
		.expect("Couldn't create java string!");
	output.into_raw()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_hey(mut env: JNIEnv, class: JClass, name: JString) {
	let sdl_context = sdl3::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();

	let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
		.position_centered()
		.build()
		.unwrap();

	let mut canvas = window.into_canvas();

	canvas.set_draw_color(Color::RGB(0, 255, 255));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut i = 0;
	'running: loop {
		i = (i + 1) % 255;
		canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
		canvas.clear();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				_ => {}
			}
		}
		// The rest of the game loop goes here...

		canvas.present();
		std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}

	unsafe { ode_sys::dInitODE2(0); }
}
