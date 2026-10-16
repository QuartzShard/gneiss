#![no_std]
#![no_main]
//! Two windows and the navigation between them.
//!
//! - `HOME` is a static window, pushed from `main`. Its persistent data `D` owns the detail
//!   window, and counts how often it has been opened.
//! - Select on home pushes the detail window from a click handler, through the `UiQueue` it is given,
//!   with `push_with`: the detail window's `appear` gets a starting count. The push runs once the
//!   handler returns.
//! - On the detail window, Up and Down change that count, held by its layout. Select pops it, as
//!   does Back by default.
//! - Home's `appear` runs whenever it is back on top, and redraws the count from its data.
//! - A long press of Select on home clears the count. One handler serves both kinds of press on
//!   that button, matching on the `Click` it gets.
extern crate alloc;

use core::fmt::Write;

use gneiss::ui::{
	Button, Click, ClickKinds, HeapText, Layer, Layout, StaticWindow, TextLayer, UiGuard, UiQueue,
	Window, WindowCtx,
};

static HOME: StaticWindow<Home> = StaticWindow::new();

/// Lives as long as the home window does.
struct HomeData {
	detail: Window<Detail>,
	opened: u32,
}

struct Home {
	text: TextLayer<HeapText>,
}

impl Home {
	fn show(&mut self, opened: u32) {
		self.text.text_mut(|text| {
			text.clear();
			let _ = write!(
				text,
				"Home\n\nSelect opens the detail window, hold it to reset.\n\nOpened {opened} times."
			);
		});
	}
}

impl Layout for Home {
	type Data = HomeData;

	fn load(ctx: &mut WindowCtx<'_, HomeData>) -> Self {
		let mut root = ctx.root();
		let mut text = TextLayer::new(root.bounds()).expect("home text layer");
		root.add_child(&mut text);
		Self { text }
	}

	fn appear(&mut self, ctx: &mut WindowCtx<'_, HomeData>, _arg: Option<()>) {
		let opened = ctx.data().map_or(0, |data| data.opened);
		self.show(opened);
	}
}

/// Rebuilt every time the detail window is pushed; `appear` sets the count it starts from.
struct Detail {
	text:  TextLayer<HeapText>,
	count: i32,
}

impl Detail {
	fn show(&mut self) {
		let count = self.count;
		self.text.text_mut(|text| {
			text.clear();
			let _ = write!(
				text,
				"Detail\n\nUp/Down: {count}\n\nSelect or Back goes home."
			);
		});
	}
}

impl Layout for Detail {
	/// The count to start from.
	type Arg = i32;

	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		let mut text = TextLayer::new(root.bounds()).expect("detail text layer");
		root.add_child(&mut text);
		Self { text, count: 0 }
	}

	fn appear(&mut self, _ctx: &mut WindowCtx<'_>, start: Option<i32>) {
		if let Some(start) = start {
			self.count = start;
		}
		self.show();
	}
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	// Handlers are set up before the windows are stored away: afterwards nothing outside their
	// own callbacks can reach them mutably.
	let mut detail = Window::<Detail>::new(None).expect("detail window");
	detail.set_click_handler(ui, Button::Up, ClickKinds::SINGLE, |detail, _, _, _| {
		detail.count += 1;
		detail.show();
	});
	detail.set_click_handler(ui, Button::Down, ClickKinds::SINGLE, |detail, _, _, _| {
		detail.count -= 1;
		detail.show();
	});
	detail.set_click_handler(ui, Button::Select, ClickKinds::SINGLE, |_, _, _, queue| {
		queue.pop(true);
	});

	let mut home = Window::<Home>::new(Some(HomeData { detail, opened: 0 })).expect("home window");
	home.set_click_handler(
		ui,
		Button::Select,
		ClickKinds::SINGLE | ClickKinds::LONG,
		|home, data, click, queue| {
			let Some(data) = data else { return };
			match click {
				Click::Long => {
					data.opened = 0;
					home.show(0);
				}
				_ => {
					data.opened += 1;
					queue.push_with(&data.detail, data.opened as i32 * 10, true);
				}
			}
		},
	);

	HOME.set(home);
	UiQueue::new(ui).push(&HOME, false);
}
