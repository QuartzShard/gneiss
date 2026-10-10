//! A multi-line status board: a ready-made window showing one `String` line per slot. Held in a
//! `static` so service handlers can update it.
extern crate alloc;

use alloc::string::String;

use crate::{
	resource::font::Font,
	sys,
	ui::{
		Layer, Layout, StaticWindow, TextAlignment, TextBuf, TextLayer, UiGuard, Window, WindowCtx,
	},
};

/// A status board window with `SLOTS` lines. Hold it in a `static`, [`init`](Self::init) it, push
/// it, then [`set_line`](Self::set_line) from anywhere that has a [`UiGuard`].
pub type StatusBoard<const SLOTS: usize> = StaticWindow<BoardLayout<SLOTS>, BoardLines<SLOTS>>;

/// The board's lines. They live as long as the window, loaded or not.
pub struct BoardLines<const SLOTS: usize> {
	lines: [Option<String>; SLOTS],
}

/// The board's layout: one text layer showing every line that is set, in slot order.
pub struct BoardLayout<const SLOTS: usize> {
	text: TextLayer<TextBuf>,
}

impl<const SLOTS: usize> BoardLayout<SLOTS> {
	fn render(&mut self, lines: &BoardLines<SLOTS>) {
		let render = |text: &mut TextBuf| {
			text.clear();
			for line in lines.lines.iter().flatten() {
				text.push_str(line);
				text.push_str("\n");
			}
		};
		self.text.text_mut(render);
	}
}

impl<const SLOTS: usize> Layout<BoardLines<SLOTS>> for BoardLayout<SLOTS> {
	fn load(ctx: &mut WindowCtx<'_, Self, BoardLines<SLOTS>>) -> Self {
		let mut root = ctx.root();
		let mut text = TextLayer::new(root.bounds()).expect("board text layer");
		text.set_alignment(TextAlignment::Center)
			.set_font(Font::system(sys::FONT_KEY_GOTHIC_18));
		root.add_child(&mut text);
		let mut board = Self { text };
		if let Some(lines) = ctx.data() {
			board.render(lines);
		}
		board
	}
}

impl<const SLOTS: usize> StatusBoard<SLOTS> {
	/// Create the board's window in this slot, ready to push.
	///
	/// # Panics
	/// If the slot is already set, or the firmware is out of memory.
	pub fn init(&self) {
		let lines = BoardLines {
			lines: [const { None }; SLOTS],
		};
		self.set(Window::new(Some(lines)).expect("board window"));
	}

	/// Replace slot `slot`'s line, redrawing if the board is loaded.
	///
	/// # Panics
	/// If `slot >= SLOTS`, or the board hasn't been [`init`](Self::init)ed.
	pub fn set_line(&self, ui: &mut UiGuard, slot: usize, line: impl Into<String>) {
		self.with(ui, |board, ctx| {
			let Some(lines) = ctx.data() else { return };
			lines.lines[slot] = Some(line.into());
			if let Some(board) = board {
				board.render(lines);
			}
		});
	}
}
