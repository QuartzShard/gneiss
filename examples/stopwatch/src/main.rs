//! A stopwatch: Select starts and stops it, Down resets it.
//!
//! The firmware's clock measures the time. A repeating [`Timer`] only redraws the display while it
//! runs, so a late frame never skews the reading.
#![no_std]
#![no_main]
extern crate alloc;

use gneiss::{
	prelude::*,
	resource::font::Font,
	sys,
	time::{self, TimeDelta, Timestamp},
	timer::Timer,
	ui::{
		Button, ClickKinds, HeapText, Layer, Layout, StaticWindow, TextAlignment, TextLayer,
		UiGuard, UiQueue, Window, WindowCtx,
		graphics::{Point, Rectangle, Size},
	},
};

static STOPWATCH: StaticWindow<Stopwatch> = StaticWindow::new();

const FRAME_MS: u32 = 30;

struct Stopwatch {
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
		let mut display = TextLayer::new(Rectangle::new(
			Point::new(0, bounds.size.height as i32 / 2 - 20),
			Size::new(bounds.size.width, 40),
		))
		.pbl_expect(c"display layer");
		display
			.set_font(Font::system(sys::FONT_KEY_GOTHIC_28_BOLD))
			.set_alignment(TextAlignment::Center);
		root.add_child(&mut display);

		let mut stopwatch = Self {
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
	fn toggle(&mut self) {
		if self.since.is_some() {
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
		if self.since.is_some() {
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
fn main(ui: &mut UiGuard) {
	let mut window = Window::<Stopwatch>::new(None).pbl_expect(c"window");
	window.set_click_handler(
		ui,
		Button::Select,
		ClickKinds::SINGLE,
		|stopwatch, _, _, _| {
			stopwatch.toggle();
		},
	);
	window.set_click_handler(
		ui,
		Button::Down,
		ClickKinds::SINGLE,
		|stopwatch, _, _, _| {
			stopwatch.reset();
		},
	);
	STOPWATCH.set(window);
	UiQueue::new(ui).push(&STOPWATCH, false);
}
