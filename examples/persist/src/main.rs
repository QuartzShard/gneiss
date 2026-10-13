//! How long the app has been open, across every launch: the running total is loaded from persistent
//! storage at launch, counted up each second, and saved when the window unloads.
#![no_std]
#![no_main]
extern crate alloc;

use gneiss::{
	SyncCell,
	resource::{
		font::Font,
		persist::{Persist, Recover},
	},
	service::TickService,
	sys,
	time::{TickTime, TimeUnits},
	ui::{
		Layer, Layout, Nav, StaticWindow, TextAlignment, TextBuf, TextLayer, UiGuard, Window,
		WindowCtx,
	},
};

gneiss::persist_keys! {
	/// Seconds open, over every launch
	SECONDS: i32,
}

static ROOT: StaticWindow<Lifetime> = StaticWindow::new();
static TOTAL: SyncCell<i32> = SyncCell::new(0);

struct Lifetime {
	text: TextLayer<TextBuf>,
}

impl Layout for Lifetime {
	fn load(ctx: &mut WindowCtx<'_, Self>) -> Self {
		let mut root = ctx.root();
		let mut text = TextLayer::new(root.bounds()).expect("text layer");
		text.set_font(Font::system(sys::FONT_KEY_GOTHIC_28_BOLD))
			.set_alignment(TextAlignment::Center);
		root.add_child(&mut text);
		let mut lifetime = Self { text };
		lifetime.show();
		lifetime
	}

	fn unload(self, _ctx: &mut WindowCtx<'_, Self>) {
		let _ = TOTAL.get().persist(SECONDS);
	}
}

impl Lifetime {
	fn show(&mut self) {
		let line = hh_mm_ss(TOTAL.get());
		self.text.text_mut(|text| {
			text.clear();
			text.push_str("App has been open for a total of\n");
			text.push_str(core::str::from_utf8(&line).unwrap_or_default());
		});
	}
}

/// `seconds` as `HH:MM:SS`, hours capped at 99.
fn hh_mm_ss(seconds: i32) -> [u8; 8] {
	let seconds = seconds.max(0) as u32;
	let parts = [(seconds / 3600).min(99), seconds / 60 % 60, seconds % 60];
	let mut out = *b"00:00:00";
	for (i, part) in parts.into_iter().enumerate() {
		out[i * 3] = b'0' + (part / 10) as u8;
		out[i * 3 + 1] = b'0' + (part % 10) as u8;
	}
	out
}

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(_time: TickTime, _changed: TimeUnits) {
	TOTAL.set(TOTAL.get() + 1);
	ROOT.with(&mut UiGuard::enter(), |lifetime, _| {
		if let Some(lifetime) = lifetime {
			lifetime.show();
		}
	});
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	TOTAL.set(i32::recover_or_default(SECONDS));
	ROOT.set(Window::new(None).expect("window"));
	Nav::new(ui).push(&ROOT, false);
}
