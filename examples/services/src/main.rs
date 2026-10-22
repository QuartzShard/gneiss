#![no_std]
#![no_main]
//! All-services demo: registers every gneiss `Service` and paints a live
//! one-line-per-service status board onto the watch via [`StatusBoard`].
//! Select turns the accelerometer and compass on and off, Up and Down make
//! the tick finer or coarser: services subscribed and unsubscribed at runtime.
//! Companion to the `resources` example.
extern crate alloc;

use core::num::NonZeroU32;

#[cfg(pbl_compass)]
use gneiss::prelude::*;
#[cfg(pbl_touch)]
use gneiss::touch;
use gneiss::{
	SyncCell, accel, battery, compass, connection, focus, health,
	link::{self, AppMessageEvent},
	service::*,
	time::{DateTime, TimeUnits},
	ui::{BoardLayout, BoardLines, Button, ClickKinds, HeapText, StatusBoard, UiGuard, UiQueue},
	wakeup,
};

#[repr(usize)]
// Fixed rows; a watch without the matching service leaves its slot empty
#[allow(dead_code)]
enum Slot {
	Controls,
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
const SLOTS: usize = 13;
static BOARD: StatusBoard<SLOTS> = StatusBoard::new();

/// The tick granularities Up and Down step through, finest first.
const TICK_UNITS: [(TimeUnits, &str); 3] = [
	(TimeUnits::SECONDS, "sec"),
	(TimeUnits::MINUTES, "min"),
	(TimeUnits::HOURS, "hour"),
];
// SAFETY: 25 isn't 0
const ACCEL_BATCH: NonZeroU32 = unsafe { NonZeroU32::new(25).unwrap_unchecked() };

/// What the buttons have set.
#[derive(Clone, Copy)]
struct Controls {
	sensors: bool,
	/// Index into [`TICK_UNITS`]
	tick:    usize,
}

impl Controls {
	fn line(self) -> HeapText {
		pbl_fmt_alloc!(
			"Sensors: {} | Tick: {}",
			if self.sensors { "on" } else { "off" },
			TICK_UNITS[self.tick].1
		)
	}
}

/// Shared by the three click handlers.
static CONTROLS: SyncCell<Controls> = SyncCell::new(Controls {
	sensors: true,
	tick:    0,
});

#[gneiss::entry]
fn main(ui: &mut UiGuard) -> UiQueue<'_> {
	BOARD.init_with(|board| {
		board.set_click_handler(
			ui,
			Button::Select,
			ClickKinds::SINGLE,
			move |board, lines, _, _| {
				let mut now = CONTROLS.get();
				now.sensors = !now.sensors;
				CONTROLS.set(now);
				set_sensors(now.sensors);
				if let Some(lines) = lines {
					let state = if now.sensors { "on" } else { "off" };
					board.set_line(
						lines,
						Slot::Accel as usize,
						pbl_fmt_alloc!("Accel: {state}"),
					);
					#[cfg(pbl_compass)]
					board.set_line(
						lines,
						Slot::Compass as usize,
						pbl_fmt_alloc!("Compass: {state}"),
					);
					show(board, lines, now);
				}
			},
		);
		for (button, finer) in [(Button::Up, true), (Button::Down, false)] {
			board.set_click_handler(ui, button, ClickKinds::SINGLE, move |board, lines, _, _| {
				let mut now = CONTROLS.get();
				now.tick = if finer {
					now.tick.saturating_sub(1)
				} else {
					(now.tick + 1).min(TICK_UNITS.len() - 1)
				};
				CONTROLS.set(now);
				TickService::subscribe(TICK_UNITS[now.tick].0);
				if let Some(lines) = lines {
					show(board, lines, now);
				}
			});
		}
	});
	BOARD.set_line(ui, Slot::Controls as usize, CONTROLS.get().line());
	let mut queue = UiQueue::new(ui);
	queue.push(&BOARD, false);
	queue
}

fn show(board: &mut BoardLayout<SLOTS>, lines: &mut BoardLines<SLOTS>, controls: Controls) {
	board.set_line(lines, Slot::Controls as usize, controls.line());
}

fn set_sensors(on: bool) {
	if on {
		AccelDataService::subscribe(ACCEL_BATCH);
		#[cfg(pbl_compass)]
		CompassService::subscribe(compass::DeltaAngle::from_degrees(10).pbl_unwrap());
	} else {
		AccelDataService::unsubscribe();
		#[cfg(pbl_compass)]
		CompassService::unsubscribe();
	}
}

// --- config + handler ---

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(time: DateTime, _changed: TimeUnits) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Tick as usize,
		pbl_fmt_alloc!(
			"Tick: {:02}:{:02}:{:02}",
			time.hour(),
			time.minute(),
			time.second()
		),
	);
}

#[gneiss::service(AccelDataService, ACCEL_BATCH)]
fn on_accel(batch: &[accel::AccelData]) {
	if let Some(s) = batch.first() {
		BOARD.set_line(
			&mut UiGuard::enter(),
			Slot::Accel as usize,
			pbl_fmt_alloc!("Accel: {},{},{}", s.x(), s.y(), s.z()),
		);
	}
}

#[cfg(pbl_compass)]
#[gneiss::service(CompassService, compass::DeltaAngle::from_degrees(10).pbl_unwrap())]
fn on_compass(heading: compass::HeadingData) {
	// The emulator's compass calls this from inside a subscribe, which Select does from its click
	// handler while the guard is held
	let Some(mut ui) = UiGuard::try_enter() else {
		return;
	};
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
			pbl_fmt_alloc!("Compass: {deg} {dir}")
		}
		_ => "Compass: unavailable".into(),
	};
	BOARD.set_line(&mut ui, Slot::Compass as usize, line);
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
		pbl_fmt_alloc!("Batt: {}%{}", charge.charge_percent, tag),
	);
}

// The frozen-SDK watches stub the backlight syscall out
#[cfg(not(pbl_sdk_frozen))]
#[gneiss::service(BacklightService)]
fn on_backlight(on: bool) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Backlight as usize,
		pbl_fmt_alloc!("Light: {}", if on { "on" } else { "off" }),
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
		pbl_fmt_alloc!("Tap: {axis}{sign}"),
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
		pbl_fmt_alloc!("Focus: {s}"),
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
		pbl_fmt_alloc!("Conn: {s}"),
	);
}

// --- wakeup ---

#[gneiss::service(WakeupService)]
fn on_wake(id: wakeup::WakeupId, cookie: wakeup::Cookie) {
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Wakeup as usize,
		pbl_fmt_alloc!("Wake: id={} cookie={}", id.0, cookie.0),
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
		pbl_fmt_alloc!("Touch: #{} {kind} ({},{})", ctx.0, event.x, event.y),
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
		pbl_fmt_alloc!("Health: #{} {s}", ctx.0),
	);
}

#[gneiss::service(AppMessageService<()>, link::SubscribeTo::RECEIVED, link::BufferSizes::default(), None)]
fn on_message(event: AppMessageEvent<'_>, _ctx: Option<&mut ()>) {
	if let AppMessageEvent::Received { inbox } = event {
		BOARD.set_line(
			&mut UiGuard::enter(),
			Slot::Message as usize,
			pbl_fmt_alloc!("Msg: {} tuples", inbox.iter().count()),
		);
	}
}
