#![no_std]
#![no_main]
//! Every kind of window, opened from one menu. The stopwatch example covers the rest: an
//! `ActionWindow` and a status bar.
//!
//! - `ROOT` is a static `MenuWindow`. Its persistent data owns the windows it opens.
//! - "Window" pushes a plain `Window` with `push_with`, which passes it a count to start from:
//!   ten times the number of times it's been opened. The row shows how many times that is. Hold
//!   Select on the row to reset it, through the `Menu`'s long click. In the window, Up and Down
//!   change the count, and repeat while held. Select or Back goes back.
//! - "Simple menu" opens a `SimpleMenuWindow`, a fixed menu in two sections with headers. The
//!   firmware counts and draws its rows from a `static` table. Selecting a fruit pushes `READER`
//!   with a few words about it.
//! - "Menu list" opens a `SimpleMenuWindow` whose rows are only known at runtime. They're a
//!   `MenuList` of lap times, built once, with every string packed into one allocation. Selecting a
//!   lap goes back.
//! - "Menu" opens level 1 of `Nested`. Every level is the same `Menu` type, so one impl serves
//!   menus whose content is only known at runtime. Rows are formatted from the level's number as
//!   they're drawn, into a `StackText`, so drawing allocates nothing. The first row opens the next
//!   level. Selecting any other row removes it, and queues a reload so the menu re-reads its rows.
//! - A menu that opened a level frees it in its layout's `appear`. Once the menu is back on top,
//!   the level has been popped and unloaded.
//! - "Scroll window" pushes `READER`, a `ScrollWindow`, with this description. Its layout sizes
//!   the content to fit the text, and Select scrolls back to the top. On round screens, the text
//!   flows around the screen's edge, and Up and Down scroll a page at a time.
//! - Back pops any window. Menus don't override it.
extern crate alloc;

use core::ffi::CStr;

use gneiss::{
	prelude::*,
	resource::font::Font,
	sys,
	ui::{
		Button, ClickKinds, HeapText, Layer, Layout, Menu, MenuCanvas, MenuIndex, MenuItem,
		MenuList, MenuListFill, MenuSection, MenuWindow, ScrollWindow, SimpleMenuWindow, StackText,
		StaticWindow, TextAlignment, TextLayer, UiGuard, UiQueue, Window, WindowCtx,
		graphics::{GContext, Point, Rectangle, Size},
	},
};

static ROOT: StaticWindow<Root> = StaticWindow::new();
static READER: StaticWindow<Reader> = StaticWindow::new();

const ABOUT: &CStr =
	c"Every kind of window gneiss has, opened from a menu of them.\n\nWindow: a plain \
	window, with click handlers of its own.\n\nSimple menu: a menu the firmware draws from a \
	table.\n\nMenu list: the same, built at runtime.\n\nMenu: a menu that draws its own rows, \
	and opens more of itself.\n\nScroll window: this text, which scrolls.\n\nThe stopwatch \
	example has the last kind, a window with an action bar.";

/// The space above and below a window's text. It's larger on round screens, which cut off the
/// corners.
const MARGIN: i32 = if cfg!(pbl_round) { 20 } else { 4 };

/// How lines align. Round screens centre them, since the lines flow around the edge.
const ALIGNMENT: TextAlignment = if cfg!(pbl_round) {
	TextAlignment::Center
} else {
	TextAlignment::Left
};

/// A text layer `height` tall, as wide as `bounds` less a 4 pixel margin on each side.
fn text_layer<S: core::ops::Deref<Target = CStr>>(bounds: Rectangle, height: u32) -> TextLayer<S> {
	let frame = Rectangle::new(
		Point::new(4, MARGIN),
		Size::new(bounds.size.width - 8, height),
	);
	let mut text = TextLayer::new(frame).pbl_expect(c"text layer");
	text.set_font(Font::system(sys::FONT_KEY_GOTHIC_24))
		.set_alignment(ALIGNMENT);
	text
}

/// A plain window's layout. It's rebuilt each time the window is pushed, and `appear` sets the
/// count it starts from.
struct Plain {
	text:  TextLayer<HeapText>,
	count: i32,
}

impl Plain {
	fn show(&mut self) {
		let count = self.count;
		self.text.text_mut(|text| {
			text.clear();
			let _ = pbl_fmt_write!(text, "Up/Down: {count}\n\nSelect or Back goes back.");
		});
	}
}

impl Layout for Plain {
	/// The count to start from.
	type Arg = i32;

	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		let bounds = root.bounds();
		let mut text = text_layer(bounds, bounds.size.height - 2 * MARGIN as u32);
		root.add_child(&mut text);
		#[cfg(pbl_round)]
		text.enable_screen_text_flow(8);
		Self { text, count: 0 }
	}

	fn appear(&mut self, _ctx: &mut WindowCtx<'_>, start: Option<i32>) {
		if let Some(start) = start {
			self.count = start;
		}
		self.show();
	}
}

/// Shows text to read. Push it with the text.
struct Reader {
	text: TextLayer,
}

impl Layout for Reader {
	type Arg = &'static CStr;

	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		// Tall enough for any text, until it's measured
		let mut text = text_layer(root.bounds(), 2000);
		root.add_child(&mut text);
		Self { text }
	}

	fn appear(&mut self, ctx: &mut WindowCtx<'_>, text: Option<&'static CStr>) {
		let Some(text) = text else { return };
		self.text.set_text(text);
		#[cfg(pbl_round)]
		self.text.enable_screen_text_flow(8);
		let frame = self.text.frame();
		let height = self.text.content_size().height + MARGIN as u32;
		self.text.set_frame(Rectangle::new(
			frame.top_left,
			Size::new(frame.size.width, height),
		));
		if let Some(scroll) = ctx.scroll() {
			scroll.fit_content([self.text.frame()]);
		}
	}
}

struct Root;

/// Lives as long as the root menu does.
struct RootData {
	plain:       Window<Plain>,
	/// How many times `plain` has been opened
	opened:      u32,
	simple_menu: SimpleMenuWindow<&'static [MenuSection<'static>; 2]>,
	menu_list:   SimpleMenuWindow<MenuList>,
	/// Level 1 of `Nested`, while it's open
	nested:      Option<MenuWindow<Nested>>,
}

/// Its own layout. It has no layers, but frees level 1 once level 1 is popped.
impl Layout for Root {
	type Data = RootData;

	fn load(_ctx: &mut WindowCtx<'_, RootData>) -> Self {
		Self
	}

	fn appear(&mut self, ctx: &mut WindowCtx<'_, RootData>, _arg: Option<()>) {
		if let Some(data) = ctx.data() {
			data.nested = None;
		}
	}
}

/// Each row's title and subtitle. The first row draws a count in place of its subtitle.
const ROOT_ROWS: [(&CStr, &CStr); 5] = [
	(c"Window", c""),
	(c"Simple menu", c"Sections from a table"),
	(c"Menu list", c"Built at runtime"),
	(c"Menu", c"Menus all the way down"),
	(c"Scroll window", c"About this example"),
];

#[gneiss::menu]
impl Menu for Root {
	type Data = RootData;
	type Layout = Self;

	fn get_num_rows(_data: &RootData, _section: u16) -> u16 {
		ROOT_ROWS.len() as u16
	}

	fn draw_row(data: &RootData, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {
		let (title, subtitle) = ROOT_ROWS[usize::from(index.row)];
		if index.row == 0 {
			let mut opened = StackText::<24>::new();
			let _ = pbl_fmt_write!(opened, "Opened {} times", data.opened);
			cell.draw_basic(ctx, title, Some(&opened));
		} else {
			cell.draw_basic(ctx, title, Some(subtitle));
		}
	}

	fn select_click<'n>(
		_layout: &'n mut Self,
		data: &'n mut RootData,
		index: MenuIndex,
		queue: &mut UiQueue<'n>,
	) {
		match index.row {
			0 => {
				data.opened += 1;
				queue.push_with(&data.plain, data.opened as i32 * 10, true);
			}
			1 => queue.push(&data.simple_menu, true),
			2 => queue.push(&data.menu_list, true),
			3 => {
				if let Some(level) = open_level(&mut data.nested, 1) {
					queue.push(level, true);
				}
			}
			_ => queue.push_with(&READER, ABOUT, true),
		}
	}

	fn select_long_click<'n>(
		_layout: &'n mut Self,
		data: &'n mut RootData,
		index: MenuIndex,
		queue: &mut UiQueue<'n>,
	) {
		if index.row == 0 {
			data.opened = 0;
			queue.reload();
		}
	}
}

/// Every level of the menu of menus.
struct Nested;

struct Level {
	number: u16,
	/// Items left: level `number` starts with `number` of them
	items:  u16,
	/// The next level, while it's open
	next:   Option<MenuWindow<Nested>>,
}

/// Build level `number` into `slot` and return it, or `None` if out of memory.
fn open_level(slot: &mut Option<MenuWindow<Nested>>, number: u16) -> Option<&MenuWindow<Nested>> {
	let level = MenuWindow::new(Level {
		number,
		items: number,
		next: None,
	})?;
	Some(slot.insert(level))
}

/// Its own layout, as for `Root`.
impl Layout for Nested {
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
impl Menu for Nested {
	type Data = Level;
	type Layout = Self;

	fn get_num_rows(level: &Level, _section: u16) -> u16 {
		level.items + 1
	}

	fn draw_row(level: &Level, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {
		let mut text = StackText::<24>::new();
		if index.row == 0 {
			let _ = pbl_fmt_write!(text, "Opens level {}", level.number + 1);
			cell.draw_basic(ctx, c"Deeper", Some(&text));
		} else {
			let _ = pbl_fmt_write!(text, "Item {}.{}", level.number, index.row);
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
	let _ = pbl_fmt_write!(title, "Lap {}", lap + 1);
	let mut time = StackText::<12>::new();
	let _ = pbl_fmt_write!(
		time,
		"{}:{:02}.{:03}",
		ms / 60_000,
		ms / 1000 % 60,
		ms % 1000
	);
	list.item(&title, Some(&time));
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) -> UiQueue<'_> {
	// Set up handlers before storing the windows away. Afterwards, only their own callbacks can
	// reach them mutably.
	let mut reader = ScrollWindow::<Reader>::new(None).pbl_expect(c"reader");
	reader.set_click_handler(ui, ClickKinds::SINGLE, |_, _, scroll, _, _| {
		scroll.set_content_offset(Point::zero(), true);
	});
	READER.set(reader);

	let mut plain = Window::<Plain>::new(None).pbl_expect(c"plain window");
	plain.set_click_handler(ui, Button::Up, ClickKinds::REPEATING, |plain, _, _, _| {
		plain.count += 1;
		plain.show();
	});
	plain.set_click_handler(ui, Button::Down, ClickKinds::REPEATING, |plain, _, _, _| {
		plain.count -= 1;
		plain.show();
	});
	plain.set_click_handler(ui, Button::Select, ClickKinds::SINGLE, |_, _, _, queue| {
		queue.pop(true);
	});

	let simple_menu = SimpleMenuWindow::new(&FRUIT, (), |_, index, queue| {
		let about = ABOUT_FRUIT[usize::from(index.section)][usize::from(index.row)];
		queue.push_with(&READER, about, true);
	})
	.pbl_expect(c"simple menu");
	let menu_list = SimpleMenuWindow::new(laps().pbl_expect(c"laps list"), (), |_, _, queue| {
		queue.pop(true);
	})
	.pbl_expect(c"menu list");
	ROOT.set(
		MenuWindow::<Root>::new(RootData {
			plain,
			opened: 0,
			simple_menu,
			menu_list,
			nested: None,
		})
		.pbl_expect(c"root menu"),
	);
	let mut queue = UiQueue::new(ui);
	queue.push(&ROOT, false);
	queue
}
