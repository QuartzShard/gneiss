//! A multi-line status board: a ready-made window showing one line of text per slot. Held in a
//! `static` so service handlers can update it.
use crate::{
	fail::PblUnwrap,
	resource::font::Font,
	sys,
	ui::{
		HeapText, Layer, Layout, StaticWindow, TextAlignment, TextLayer, UiGuard, Window, WindowCtx,
	},
};

/// A status board window with `SLOTS` lines. Hold it in a `static`, [`init`](Self::init) it, push
/// it, then [`set_line`](Self::set_line) from anywhere that has a [`UiGuard`].
pub type StatusBoard<const SLOTS: usize> = StaticWindow<BoardLayout<SLOTS>>;

/// The board's lines. They live as long as the window, loaded or not.
pub struct BoardLines<const SLOTS: usize> {
	lines: [Option<HeapText>; SLOTS],
}

/// The board's layout: one text layer showing every line that is set, in slot order.
pub struct BoardLayout<const SLOTS: usize> {
	text: TextLayer<HeapText>,
}

impl<const SLOTS: usize> BoardLayout<SLOTS> {
	/// Replace slot `slot`'s line and redraw, from one of the board's own callbacks, such as a click
	/// handler.
	///
	/// # Panics
	/// If `slot >= SLOTS`.
	pub fn set_line(
		&mut self,
		lines: &mut BoardLines<SLOTS>,
		slot: usize,
		line: impl Into<HeapText>,
	) {
		lines.lines[slot] = Some(line.into());
		self.render(lines);
	}

	fn render(&mut self, lines: &BoardLines<SLOTS>) {
		let render = |text: &mut HeapText| {
			text.clear();
			for line in lines.lines.iter().flatten() {
				text.push_bytes(line.as_c_str().to_bytes());
				text.push_str("\n");
			}
		};
		self.text.text_mut(render);
	}
}

impl<const SLOTS: usize> Layout for BoardLayout<SLOTS> {
	type Data = BoardLines<SLOTS>;

	fn load(ctx: &mut WindowCtx<'_, BoardLines<SLOTS>>) -> Self {
		let mut root = ctx.root();
		let mut text =
			crate::fail::gneiss_expect!(TextLayer::new(root.bounds()), "board text layer");
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
	#[track_caller]
	pub fn init(&self) {
		self.init_with(|_| {});
	}

	/// [`init`](Self::init), letting `configure` set the window up first, such as its click
	/// handlers.
	///
	/// # Panics
	/// If the slot is already set, or the firmware is out of memory.
	#[track_caller]
	pub fn init_with(&self, configure: impl FnOnce(&mut Window<BoardLayout<SLOTS>>)) {
		let lines = BoardLines {
			lines: [const { None }; SLOTS],
		};
		let mut window = Window::new(Some(lines)).pbl_expect(c"board window");
		configure(&mut window);
		self.set(window);
	}

	/// Replace slot `slot`'s line, redrawing if the board is loaded.
	///
	/// # Panics
	/// If `slot >= SLOTS`, or the board hasn't been [`init`](Self::init)ed.
	#[track_caller]
	pub fn set_line(&self, ui: &mut UiGuard, slot: usize, line: impl Into<HeapText>) {
		self.with(ui, |board, ctx| {
			let Some(lines) = ctx.data() else { return };
			match board {
				Some(board) => board.set_line(lines, slot, line),
				None => lines.lines[slot] = Some(line.into()),
			}
		});
	}
}
