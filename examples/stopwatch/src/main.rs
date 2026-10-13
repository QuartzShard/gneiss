//! A stopwatch: Select starts and stops it, Down resets it.
//!
//! The firmware's clock measures the time. A repeating [`Timer`] only redraws the display while it
//! runs, so a late frame never skews the reading.
#![no_std]
#![no_main]
extern crate alloc;

use core::fmt::Write;

use gneiss::{
	resource::font::Font,
	service::TickService,
	sys,
	time::{FineDuration, FineTime},
	timer::Timer,
	ui::{
		Button, ClickKinds, Layer, Layout, Nav, StaticWindow, TextAlignment, TextBuf, TextLayer,
		UiGuard, Window, WindowCtx,
		graphics::{Point, Rectangle, Size},
	},
};

static STOPWATCH: StaticWindow<Stopwatch> = StaticWindow::new();

const FRAME_MS: u32 = 30;

struct Stopwatch {
	display: TextLayer<TextBuf>,
	/// Time from earlier runs.
	banked:  FineDuration,
	/// When the current run started.
	since:   Option<FineTime>,
	/// Redraws the display while running. Dropping it, here or when the window unloads, stops it.
	redraw:  Option<Timer>,
}

impl Layout for Stopwatch {
	fn load(ctx: &mut WindowCtx<'_, Self>) -> Self {
		let mut root = ctx.root();
		let bounds = root.bounds();
		let mut display = TextLayer::new(Rectangle::new(
			Point::new(0, bounds.size.height as i32 / 2 - 20),
			Size::new(bounds.size.width, 40),
		))
		.expect("display layer");
		display
			.set_font(Font::system(sys::FONT_KEY_GOTHIC_28_BOLD))
			.set_alignment(TextAlignment::Center);
		root.add_child(&mut display);

		let mut stopwatch = Self {
			display,
			banked: FineDuration::ZERO,
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
		self.since = Some(TickService::fine());
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
			self.banked += TickService::fine() - since;
		}
		self.redraw = None;
		self.show();
	}

	fn reset(&mut self) {
		self.banked = FineDuration::ZERO;
		if self.since.is_some() {
			self.since = Some(TickService::fine());
		}
		self.show();
	}

	fn show(&mut self) {
		let running = self
			.since
			.map_or(FineDuration::ZERO, |since| TickService::fine() - since);
		let ms = (self.banked + running).as_millis();
		self.display.text_mut(|text| {
			text.clear();
			let _ = write!(text, "{}.{:03}", ms / 1000, ms % 1000);
		});
	}
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	let mut window = Window::<Stopwatch>::new(None).expect("window");
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
	Nav::new(ui).push(&STOPWATCH, false);
}
