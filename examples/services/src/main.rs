#![no_std]
#![no_main]
//! All-services demo: registers every gneiss `Service` and paints a live
//! one-line-per-service status board onto the watch. Companion to the
//! `resources` example. Needs gneiss's `alloc` feature.
extern crate alloc;

use alloc::format;

use gneiss::{
	accel, compass, connection, focus, health,
	service::*,
	sys,
	time::{TickTime, TimeUnits},
	touch, wakeup,
};

/// A single-app-task status board painted into one `TextLayer`. Each service
/// owns one line; a handler updates its line and the whole board re-renders.
///
/// The layer and its backing string must outlive `main` (its frame is gone
/// before the event loop runs), so they live in a `static`. Sound because the
/// app is a single cooperative task: handlers run one at a time in the event
/// loop, never reentrant, so there is never an overlapping `&mut`.
mod screen {
	use alloc::{ffi::CString, string::String};
	use core::{cell::UnsafeCell, ptr};

	use crate::sys;

	#[repr(usize)]
	#[derive(Clone, Copy)]
	pub enum Slot {
		Tick,
		Battery,
		Backlight,
		Tap,
		Accel,
		Compass,
		Focus,
		Connection,
		Wakeup,
		Touch,
		Health,
	}
	const SLOTS: usize = 11;

	struct Inner {
		layer:   *mut sys::TextLayer,
		lines:   [Option<String>; SLOTS],
		// Keeps the currently-displayed C string alive: the text layer stores
		// the pointer, not a copy, so it must outlive the next render.
		current: Option<CString>,
	}

	struct Board(UnsafeCell<Inner>);
	// SAFETY: only ever touched from the single app task (see module docs).
	unsafe impl Sync for Board {}

	static BOARD: Board = Board(UnsafeCell::new(Inner {
		layer:   ptr::null_mut(),
		lines:   [const { None }; SLOTS],
		current: None,
	}));

	pub fn init(layer: *mut sys::TextLayer) {
		// SAFETY: single-task, non-reentrant; see module docs.
		unsafe { &mut *BOARD.0.get() }.layer = layer;
	}

	pub fn set(slot: Slot, line: String) {
		// SAFETY: single-task, non-reentrant; see module docs.
		let inner = unsafe { &mut *BOARD.0.get() };
		if inner.layer.is_null() {
			return;
		}
		inner.lines[slot as usize] = Some(line);

		let mut text = String::new();
		for line in inner.lines.iter().flatten() {
			text.push_str(line);
			text.push('\n');
		}
		if let Ok(cs) = CString::new(text) {
			// Point the layer at the new buffer BEFORE dropping the old one.
			// SAFETY: valid layer, `cs` outlives this (moved into `current`).
			unsafe { sys::text_layer_set_text(inner.layer, cs.as_ptr()) };
			inner.current = Some(cs); // drops the previous CString, now unreferenced
		}
	}
}
use screen::Slot;

#[gneiss::entry]
fn main() {
	unsafe {
		let window = sys::window_create();
		let root = sys::window_get_root_layer(window);
		let bounds = sys::layer_get_bounds(root);
		let text = sys::text_layer_create(bounds);
		sys::text_layer_set_text_alignment(text, sys::GTextAlignment_GTextAlignmentCenter);
		sys::text_layer_set_font(
			text,
			sys::fonts_get_system_font(sys::FONT_KEY_GOTHIC_24.as_ptr()),
		);
		sys::text_layer_set_text(text, c"waiting for services...".as_ptr());
		sys::layer_add_child(root, sys::text_layer_get_layer(text));
		sys::window_stack_push(window, false);
		screen::init(text);
	}
}

// --- config + handler ---

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(time: TickTime, _changed: TimeUnits) {
	screen::set(
		Slot::Tick,
		format!(
			"Tick: {:02}:{:02}:{:02}",
			time.tm_hour, time.tm_min, time.tm_sec
		),
	);
}

#[gneiss::service(AccelDataService, core::num::NonZeroU32::new(25).unwrap())]
fn on_accel(batch: &[accel::AccelData]) {
	if let Some(s) = batch.first() {
		screen::set(Slot::Accel, format!("Accel: {},{},{}", s.x(), s.y(), s.z()));
	}
}

#[gneiss::service(CompassService, compass::DeltaAngle::from_degrees(10).unwrap())]
fn on_compass(heading: compass::HeadingData) {
	let line = match heading.status() {
		compass::CompassStatus::Calibrated | compass::CompassStatus::Calibrating => {
			let deg = heading.heading().as_degrees_cw();
			let dir = match deg {
				0..45 | 315..360 => "N",
				45..135 => "E",
				135..225 => "S",
				225..315 => "W",
				_ => "?",
			};
			format!("Compass: {deg} {dir}")
		}
		_ => "Compass: unavailable".into(),
	};
	screen::set(Slot::Compass, line);
}

// --- handler only ---

#[gneiss::service(BatteryService)]
fn on_battery(charge: sys::BatteryChargeState) {
	let tag = if charge.is_charging {
		" chg"
	} else if charge.is_plugged {
		" plug"
	} else {
		""
	};
	screen::set(
		Slot::Battery,
		format!("Batt: {}%{}", charge.charge_percent, tag),
	);
}

#[gneiss::service(BacklightService)]
fn on_backlight(on: bool) {
	screen::set(
		Slot::Backlight,
		format!("Light: {}", if on { "on" } else { "off" }),
	);
}

#[gneiss::service(AccelTapService)]
fn on_tap(axis: accel::Axis, dir: accel::Direction) {
	let axis = match axis {
		accel::Axis::X => "X",
		accel::Axis::Y => "Y",
		accel::Axis::Z => "Z",
	};
	let sign = match dir {
		accel::Direction::Positive => "+",
		accel::Direction::Negative => "-",
	};
	screen::set(Slot::Tap, format!("Tap: {axis}{sign}"));
}

// --- bundles (opt into slots via the SubscribeTo flags) ---

#[gneiss::service(AppFocusService, focus::SubscribeTo::WILL | focus::SubscribeTo::DID)]
fn on_focus(event: focus::Event) {
	let s = match event {
		focus::Event::Gaining => "will-gain",
		focus::Event::Losing => "will-lose",
		focus::Event::Gained => "did-gain",
		focus::Event::Lost => "did-lose",
	};
	screen::set(Slot::Focus, format!("Focus: {s}"));
}

#[gneiss::service(ConnectionService, connection::SubscribeTo::APP | connection::SubscribeTo::PEBBLEKIT)]
fn on_connection(event: connection::Event) {
	let s = match event {
		connection::Event::AppConnected => "phone up",
		connection::Event::AppDisconnected => "phone down",
		connection::Event::PebbleKitConnected => "kit up",
		connection::Event::PebbleKitDisconnected => "kit down",
	};
	screen::set(Slot::Connection, format!("Conn: {s}"));
}

// --- wakeup ---

#[gneiss::service(WakeupService)]
fn on_wake(id: wakeup::WakeupId, cookie: wakeup::Cookie) {
	screen::set(
		Slot::Wakeup,
		format!("Wake: id={} cookie={}", id.0, cookie.0),
	);
}

// --- context-bearing (pass Some(ctx); handler gets Option<&mut ctx>) ---

struct TouchCtx(u32);
#[gneiss::service(TouchService<TouchCtx>, Some(TouchCtx(0)))]
fn on_touch(event: touch::Event, ctx: Option<&mut TouchCtx>) {
	let Some(ctx) = ctx else { return };
	ctx.0 += 1;
	let kind = match event.event_type {
		touch::EventType::TouchDown => "down",
		touch::EventType::LiftOff => "up",
		touch::EventType::PositionUpdate => "move",
	};
	screen::set(
		Slot::Touch,
		format!("Touch: #{} {kind} ({},{})", ctx.0, event.x, event.y),
	);
}

struct HealthCtx(u32);
#[gneiss::service(HealthService<HealthCtx>, Some(HealthCtx(0)))]
fn on_health(event: health::Event, ctx: Option<&mut HealthCtx>) {
	let Some(ctx) = ctx else { return };
	ctx.0 += 1;
	let s = match event {
		health::Event::Significant => "significant",
		health::Event::Movement => "movement",
		health::Event::Sleep => "sleep",
		health::Event::HeartRate => "heart-rate",
		health::Event::HRV => "hrv",
	};
	screen::set(Slot::Health, format!("Health: #{} {s}", ctx.0));
}
