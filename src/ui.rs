/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use sdl3::{AudioSubsystem, EventPump, EventSubsystem, GamepadSubsystem, HapticSubsystem, JoystickSubsystem, Sdl, VideoSubsystem};
use crate::{FerriciaError, FerriciaResult};

pub(crate) mod rendering;
pub(crate) mod window;

pub(crate) struct SdlHandle {
	audio: AudioSubsystem,
	events: EventSubsystem,
	joystick: JoystickSubsystem,
	haptic: HapticSubsystem,
	gamepad: GamepadSubsystem,
	video: VideoSubsystem,
	event_pump: EventPump,
	sdl_context: Sdl,
}

impl From<sdl3::Error> for FerriciaError {
	fn from(value: sdl3::Error) -> Self {
		value.to_string().into()
	}
}

impl From<sdl3::IntegerOrSdlError> for FerriciaError {
	fn from(value: sdl3::IntegerOrSdlError) -> Self {
		value.to_string().into()
	}
}

pub(crate) fn init_sdl_handle() -> FerriciaResult<SdlHandle> {
	let sdl_context = sdl3::init()?;
	Ok(SdlHandle {
		audio: sdl_context.audio()?,
		events: sdl_context.event()?,
		joystick: sdl_context.joystick()?,
		haptic: sdl_context.haptic()?,
		gamepad: sdl_context.gamepad()?,
		video: sdl_context.video()?,
		event_pump: sdl_context.event_pump()?,
		sdl_context,
	})
}
