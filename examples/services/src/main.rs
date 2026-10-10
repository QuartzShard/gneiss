#![no_std]
#![no_main]
//! All-services demo: registers every gneiss `Service` and paints a live
//! one-line-per-service status board onto the watch via [`StatusBoard`].
//! Companion to the `resources` example. Needs gneiss's `alloc` feature.
extern crate alloc;

use alloc::format;

#[cfg(pbl_touch)]
use gneiss::touch;
use gneiss::{
	accel, battery, compass, connection, focus, health,
	link::{self, AppMessageEvent},
	service::*,
	sys,
	time::{TickTime, TimeUnits},
	ui::{Nav, StatusBoard, UiGuard},
	wakeup,
};

#[repr(usize)]
// Fixed rows; a watch without the matching service leaves its slot empty
#[allow(dead_code)]
enum Slot {
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
	Message,
}
static BOARD: StatusBoard<12> = StatusBoard::new();

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	BOARD.init();
	BOARD.set_line(ui, 0, "waiting for services...");
	Nav::new(ui).push(&BOARD, false);
}

// --- config + handler ---

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(time: TickTime, _changed: TimeUnits) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Tick as usize,
		format!(
			"Tick: {:02}:{:02}:{:02}",
			time.tm_hour, time.tm_min, time.tm_sec
		),
	);
}

#[gneiss::service(AccelDataService, core::num::NonZeroU32::new(25).unwrap())]
fn on_accel(batch: &[accel::AccelData]) {
	if let Some(s) = batch.first() {
		BOARD.set_line(
			&mut UiGuard::enter(),
			Slot::Accel as usize,
			format!("Accel: {},{},{}", s.x(), s.y(), s.z()),
		);
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
	BOARD.set_line(&mut UiGuard::enter(), Slot::Compass as usize, line);
}

// --- handler only ---

#[gneiss::service(BatteryService)]
fn on_battery(charge: battery::BatteryChargeState) {
	let tag = if charge.is_charging {
		" chg"
	} else if charge.is_plugged {
		" plug"
	} else {
		""
	};
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Battery as usize,
		format!("Batt: {}%{}", charge.charge_percent, tag),
	);
}

// The frozen-SDK watches stub the backlight syscall out
#[cfg(not(pbl_sdk_frozen))]
#[gneiss::service(BacklightService)]
fn on_backlight(on: bool) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Backlight as usize,
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
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Tap as usize,
		format!("Tap: {axis}{sign}"),
	);
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
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Focus as usize,
		format!("Focus: {s}"),
	);
}

#[gneiss::service(ConnectionService, connection::SubscribeTo::APP | connection::SubscribeTo::PEBBLEKIT)]
fn on_connection(event: connection::Event) {
	let s = match event {
		connection::Event::AppConnected => "phone up",
		connection::Event::AppDisconnected => "phone down",
		connection::Event::PebbleKitConnected => "kit up",
		connection::Event::PebbleKitDisconnected => "kit down",
	};
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Connection as usize,
		format!("Conn: {s}"),
	);
}

// --- wakeup ---

#[gneiss::service(WakeupService)]
fn on_wake(id: wakeup::WakeupId, cookie: wakeup::Cookie) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Wakeup as usize,
		format!("Wake: id={} cookie={}", id.0, cookie.0),
	);
}

// --- context-bearing (pass Some(ctx); handler gets Option<&mut ctx>) ---

#[cfg(pbl_touch)]
struct TouchCtx(u32);
#[cfg(pbl_touch)]
#[gneiss::service(TouchService<TouchCtx>, Some(TouchCtx(0)))]
fn on_touch(event: touch::Event, ctx: Option<&mut TouchCtx>) {
	let Some(ctx) = ctx else { return };
	ctx.0 += 1;
	let kind = match event.event_type {
		touch::EventType::TouchDown => "down",
		touch::EventType::LiftOff => "up",
		touch::EventType::PositionUpdate => "move",
	};
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Touch as usize,
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
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Health as usize,
		format!("Health: #{} {s}", ctx.0),
	);
}

#[gneiss::service(AppMessageService<()>, link::SubscribeTo::RECEIVED, link::BufferSizes::default(), None)]
fn on_message(event: AppMessageEvent<'_>, _ctx: Option<&mut ()>) {
	if let AppMessageEvent::Received { inbox } = event {
		BOARD.set_line(
			&mut UiGuard::enter(),
			Slot::Message as usize,
			format!("Msg: {} tuples", inbox.iter().count()),
		);
	}
}
