#![no_std]
#![no_main]
//! Menus: a root menu, a fixed sub-menu, and a menu that opens more of itself.
//!
//! - `ROOT` is a static `MenuWindow`. Its persistent data owns the menus it opens.
//! - "Deeper" opens level 1 of `Deeper`. Every level is the same `Menu`, its rows formatted from
//!   the level's number as they're drawn, into a `StackText`, so drawing allocates nothing. Its
//!   first row builds and opens the next level, so one `Menu` impl serves menus whose content is
//!   only known at runtime. Selecting an item eats it, and queues a reload so the menu re-reads
//!   its rows.
//! - Each menu that opens a level frees it in its layout's `appear`: when the menu is back on top,
//!   the level was popped, and it has already unloaded.
//! - "Fruit" opens a `SimpleMenuWindow`: a fixed menu in two sections, with headers, which the
//!   firmware counts and draws from a `static` table. Selecting a fruit pushes `DETAIL`, a plain
//!   window, with a few words about it.
//! - "Laps" opens a `SimpleMenuWindow` whose rows are only known at runtime: a `MenuList`, built
//!   once from lap times formatted into `StackText`, and packed with every string into one
//!   allocation. Selecting a lap pops back.
//! - "About" pushes the same detail window with this description.
//! - Back pops, as on any window: menus leave it alone.
extern crate alloc;

use core::{ffi::CStr, fmt::Write};

use gneiss::ui::{
	Layer, Layout, Menu, MenuCanvas, MenuIndex, MenuItem, MenuList, MenuListFill, MenuSection,
	MenuWindow, SimpleMenuWindow, StackText, StaticWindow, TextLayer, UiGuard, UiQueue, Window,
	WindowCtx, graphics::GContext,
};

static ROOT: StaticWindow<Root> = StaticWindow::new();
static DETAIL: StaticWindow<Detail> = StaticWindow::new();

const ABOUT: &CStr =
	c"Menus example\n\nEvery level of Deeper is the same Menu type, its rows built \
	from a number: a new kind of screen isn't needed for new content.";

/// Text to read, pushed with the text to show.
struct Detail {
	text: TextLayer,
}

impl Layout for Detail {
	type Arg = &'static CStr;

	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		let mut text = TextLayer::new(root.bounds()).expect("detail text layer");
		root.add_child(&mut text);
		Self { text }
	}

	fn appear(&mut self, _ctx: &mut WindowCtx<'_>, text: Option<&'static CStr>) {
		if let Some(text) = text {
			self.text.set_text(text);
		}
	}
}

struct Root;

/// Lives as long as the root menu does.
struct RootData {
	fruit:  SimpleMenuWindow<&'static [MenuSection<'static>; 2]>,
	laps:   SimpleMenuWindow<MenuList>,
	/// Level 1 of `Deeper`, while it's open
	deeper: Option<MenuWindow<Deeper>>,
}

/// Its own layout: it has no layers, but frees level 1 once it's popped.
impl Layout for Root {
	type Data = RootData;

	fn load(_ctx: &mut WindowCtx<'_, RootData>) -> Self {
		Self
	}

	fn appear(&mut self, ctx: &mut WindowCtx<'_, RootData>, _arg: Option<()>) {
		if let Some(data) = ctx.data() {
			data.deeper = None;
		}
	}
}

const ROOT_ROWS: [(&CStr, &CStr); 4] = [
	(c"Deeper", c"Menus all the way down"),
	(c"Fruit", c"Sections and headers"),
	(c"Laps", c"Built at runtime"),
	(c"About", c"What this is"),
];

#[gneiss::menu]
impl Menu for Root {
	type Data = RootData;
	type Layout = Self;

	fn get_num_rows(_data: &RootData, _section: u16) -> u16 {
		ROOT_ROWS.len() as u16
	}

	fn draw_row(_data: &RootData, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {
		let (title, subtitle) = ROOT_ROWS[usize::from(index.row)];
		cell.draw_basic(ctx, title, Some(subtitle));
	}

	fn select_click<'n>(
		_layout: &'n mut Self,
		data: &'n mut RootData,
		index: MenuIndex,
		queue: &mut UiQueue<'n>,
	) {
		match index.row {
			0 => {
				if let Some(level) = open_level(&mut data.deeper, 1) {
					queue.push(level, true);
				}
			}
			1 => queue.push(&data.fruit, true),
			2 => queue.push(&data.laps, true),
			_ => queue.push_with(&DETAIL, ABOUT, true),
		}
	}
}

/// Every level of the menu of menus.
struct Deeper;

struct Level {
	number: u16,
	/// Items left: level `number` starts with `number` of them
	items:  u16,
	/// The next level, while it's open
	next:   Option<MenuWindow<Deeper>>,
}

/// Build level `number` into `slot` and return it, or `None` if out of memory.
fn open_level(slot: &mut Option<MenuWindow<Deeper>>, number: u16) -> Option<&MenuWindow<Deeper>> {
	let level = MenuWindow::new(Level {
		number,
		items: number,
		next: None,
	})?;
	Some(slot.insert(level))
}

/// Its own layout, as for `Root`.
impl Layout for Deeper {
	type Data = Level;

	fn load(_ctx: &mut WindowCtx<'_, Level>) -> Self {
		Self
	}

	fn appear(&mut self, ctx: &mut WindowCtx<'_, Level>, _arg: Option<()>) {
		// If we appeared by popping our child, dealloc it now
		if let Some(level) = ctx.data() {
			level.next = None;
		}
	}
}

#[gneiss::menu]
impl Menu for Deeper {
	type Data = Level;
	type Layout = Self;

	fn get_num_rows(level: &Level, _section: u16) -> u16 {
		level.items + 1
	}

	fn draw_row(level: &Level, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {
		let mut text = StackText::<24>::new();
		if index.row == 0 {
			let _ = write!(text, "Opens level {}", level.number + 1);
			cell.draw_basic(ctx, c"Deeper", Some(&text));
		} else {
			let _ = write!(text, "Item {}.{}", level.number, index.row);
			cell.draw_basic(ctx, &text, None);
		}
	}

	fn select_click<'n>(
		_layout: &'n mut Self,
		level: &'n mut Level,
		index: MenuIndex,
		queue: &mut UiQueue<'n>,
	) {
		if index.row == 0 {
			let number = level.number + 1;
			if let Some(next) = open_level(&mut level.next, number) {
				queue.push(next, true);
			}
		} else {
			level.items -= 1;
			queue.reload();
		}
	}
}

/// A fixed menu in sections, counted and drawn by the firmware from this table.
static FRUIT: [MenuSection; 2] = [
	MenuSection::new(&[MenuItem::new(c"Plum"), MenuItem::new(c"Cherry")]).title(c"Stone fruit"),
	MenuSection::new(&[
		MenuItem::new(c"Strawberry").subtitle(c"Not a berry"),
		MenuItem::new(c"Blueberry"),
		MenuItem::new(c"Raspberry").subtitle(c"Not a berry either"),
	])
	.title(c"Berries"),
];

/// What selecting each fruit shows, by section and row.
const ABOUT_FRUIT: [&[&CStr]; 2] = [
	&[
		c"Plum\n\nDried, it's a prune.",
		c"Cherry\n\nA stone fruit, like the plum, the peach and the almond.",
	],
	&[
		c"Strawberry\n\nNot a berry: its seeds are on the outside.",
		c"Blueberry\n\nA true berry.",
		c"Raspberry\n\nNot a berry either: a cluster of tiny stone fruits.",
	],
];

/// Lap times in milliseconds, as if recorded while the app ran.
const LAPS_MS: [u32; 5] = [83_450, 81_920, 85_010, 80_770, 82_305];

/// The laps menu: the fastest lap, then every lap, each with its time.
fn laps() -> Option<MenuList> {
	MenuList::new(|list| {
		let (best, best_ms) = LAPS_MS
			.iter()
			.enumerate()
			.min_by_key(|&(_, ms)| ms)
			.unwrap_or((0, &0));
		list.section(Some(c"Fastest"));
		lap_row(list, best, *best_ms);
		list.section(Some(c"Laps"));
		for (lap, ms) in LAPS_MS.iter().enumerate() {
			lap_row(list, lap, *ms);
		}
	})
}

fn lap_row(list: &mut MenuListFill<'_>, lap: usize, ms: u32) {
	let mut title = StackText::<12>::new();
	let _ = write!(title, "Lap {}", lap + 1);
	let mut time = StackText::<12>::new();
	let _ = write!(
		time,
		"{}:{:02}.{:03}",
		ms / 60_000,
		ms / 1000 % 60,
		ms % 1000
	);
	list.item(&title, Some(&time));
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	DETAIL.set(Window::<Detail>::new(None).expect("detail window"));
	let fruit = SimpleMenuWindow::new(&FRUIT, (), |_, index, queue| {
		let about = ABOUT_FRUIT[usize::from(index.section)][usize::from(index.row)];
		queue.push_with(&DETAIL, about, true);
	})
	.expect("fruit menu");
	let laps = SimpleMenuWindow::new(laps().expect("laps list"), (), |_, _, queue| {
		queue.pop(true);
	})
	.expect("laps menu");
	ROOT.set(
		MenuWindow::<Root>::new(RootData {
			fruit,
			laps,
			deeper: None,
		})
		.expect("root menu"),
	);
	UiQueue::new(ui).push(&ROOT, false);
}
