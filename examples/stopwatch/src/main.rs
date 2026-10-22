//! A stopwatch: Select starts and stops it, Down resets it.
//!
//! The firmware's clock measures the time. A repeating [`Timer`] only redraws the display while it
//! runs, so a late frame never skews the reading. A [`StatusBarLayer`] along the top shows the time
//! of day, which the firmware keeps current itself.
//!
//! It's an [`ActionWindow`], so its action bar shows what Select and Down do. Click handlers get
//! the bar. Select's handler swaps the play and pause icons: it keeps the one that isn't showing,
//! and [`replace_icon_animated`](gneiss::ui::ActionBar::replace_icon_animated) hands back the one
//! it replaces. Each icon is loaded only once.
#![no_std]
#![no_main]
extern crate alloc;

use gneiss::{
	prelude::*,
	resource::{font::Font, image::GBitMap},
	sys,
	time::{self, TimeDelta, Timestamp},
	timer::Timer,
	ui::{
		ACTION_BAR_WIDTH, ActionButton, ActionClickKinds, ActionWindow, Button, HeapText, Layer,
		Layout, STATUS_BAR_HEIGHT, Separator, StaticWindow, StatusBarLayer, TextAlignment,
		TextLayer, UiGuard, UiQueue, WindowCtx,
		graphics::{Point, Rectangle, Size},
	},
};

static STOPWATCH: StaticWindow<Stopwatch> = StaticWindow::new();

const FRAME_MS: u32 = 30;

struct Stopwatch {
	_status: StatusBarLayer,
	display: TextLayer<HeapText>,
	/// Time from earlier runs.
	banked:  TimeDelta,
	/// When the current run started.
	since:   Option<Timestamp>,
	/// Redraws the display while running. Dropping it, here or when the window unloads, stops it.
	redraw:  Option<Timer>,
}

impl Layout for Stopwatch {
	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		let bounds = root.bounds();
		// What the action bar leaves of the screen
		let width = bounds.size.width - ACTION_BAR_WIDTH;

		let mut status = StatusBarLayer::new().pbl_expect(c"status bar");
		status
			.set_separator(Separator::Dotted)
			.set_frame(Rectangle::new(
				Point::zero(),
				Size::new(width, STATUS_BAR_HEIGHT),
			));
		root.add_child(&mut status);

		// Centred in the space below the status bar
		let below = (bounds.size.height + STATUS_BAR_HEIGHT) as i32 / 2;
		let mut display = TextLayer::new(Rectangle::new(
			Point::new(0, below - 20),
			Size::new(width, 40),
		))
		.pbl_expect(c"display layer");
		display
			.set_font(Font::system(sys::FONT_KEY_GOTHIC_28_BOLD))
			.set_alignment(TextAlignment::Center);
		root.add_child(&mut display);

		let mut stopwatch = Self {
			_status: status,
			display,
			banked: TimeDelta::ZERO,
			since: None,
			redraw: None,
		};
		stopwatch.show();
		stopwatch
	}
}

impl Stopwatch {
	fn running(&self) -> bool {
		self.since.is_some()
	}

	fn toggle(&mut self) {
		if self.running() {
			self.stop();
		} else {
			self.start();
		}
	}

	fn start(&mut self) {
		self.since = Some(time::now());
		self.redraw = Some(Timer::repeating(FRAME_MS, || {
			STOPWATCH.with(&mut UiGuard::enter(), |stopwatch, _| {
				if let Some(stopwatch) = stopwatch {
					stopwatch.show();
				}
			});
			// Returning `Some(timeout_ms)` runs it again after that long, `None` stops it
			Some(FRAME_MS)
		}));
	}

	fn stop(&mut self) {
		if let Some(since) = self.since.take() {
			self.banked += time::now() - since;
		}
		self.redraw = None;
		self.show();
	}

	fn reset(&mut self) {
		self.banked = TimeDelta::ZERO;
		if self.running() {
			self.since = Some(time::now());
		}
		self.show();
	}

	fn show(&mut self) {
		let running = self
			.since
			.map_or(TimeDelta::ZERO, |since| time::now() - since);
		let ms = (self.banked + running).as_millis();
		self.display.text_mut(|text| {
			text.clear();
			let _ = pbl_fmt_write!(text, "{}.{:03}", ms / 1000, ms % 1000);
		});
	}
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) -> UiQueue<'_> {
	let icon = |key| GBitMap::load(key).pbl_expect(c"icon");

	let mut window = ActionWindow::<Stopwatch>::new(None).pbl_expect(c"window");
	window
		.action_bar(ui)
		.set_icon(ActionButton::Select, icon(resources::ICON_PLAY))
		.set_icon(ActionButton::Down, icon(resources::ICON_RESET));
	// The Select icon that isn't showing: pause while stopped, play while running
	let mut spare = Some(icon(resources::ICON_PAUSE));
	window.set_click_handler(
		ui,
		Button::Select,
		ActionClickKinds::SINGLE,
		move |stopwatch, _, bar, _, _| {
			stopwatch.toggle();
			if let Some(icon) = spare.take() {
				spare = bar.replace_icon_animated(ActionButton::Select, icon);
			}
		},
	);
	window.set_click_handler(
		ui,
		Button::Down,
		ActionClickKinds::SINGLE,
		|stopwatch, _, _, _, _| {
			stopwatch.reset();
		},
	);
	STOPWATCH.set(window);
	let mut queue = UiQueue::new(ui);
	queue.push(&STOPWATCH, false);
	queue
}
