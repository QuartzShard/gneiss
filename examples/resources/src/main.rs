//! Declares every currently-supported resource kind (see Cargo.toml) and shows
//! how each is loaded at runtime. The generated `resources` module gives one
//! typed key per declared resource, named after the declaration: the key's type
//! follows the declared `type`.
#![no_std]
#![no_main]
extern crate alloc;

use alloc::{ffi::CString, vec::Vec};

use gneiss::{
	SyncCell,
	alloc::PebbleAlloc,
	prelude::*,
	resource::{
		font::Font,
		image::{DrawCommand, DrawCommandSequence, GBitMap},
		raw,
	},
	sys,
	timer::Timer,
	ui::{
		BitmapLayer, DataLayer, Layer, Layout, StaticWindow, TextLayer, UiGuard, UiQueue, Window,
		WindowCtx,
		graphics::{Colour, GContext, Point, Rectangle, Size},
	},
};

static ROOT: StaticWindow<Resources> = StaticWindow::new();

/// Matches the `spinner` resource's `frameDurationMs`.
const SPINNER_STEP_MS: u32 = 60;

// Elapsed time into the spinner's sequence, advanced by the layout's timer.
static SPINNER_MS: SyncCell<u32> = SyncCell::new(0);

/// Everything on screen, created when the window loads and dropped when it unloads. Each layer owns
/// the resource it shows, so nothing needs leaking.
struct Resources {
	_title:         TextLayer<CString>,
	_image:         BitmapLayer,
	_pbi:           BitmapLayer,
	_ferris:        DataLayer<DrawCommand>,
	spinner:        DataLayer<DrawCommandSequence>,
	// Dropping the timer cancels it, so it stops when the window unloads
	_spinner_timer: Timer,
}

impl Layout for Resources {
	fn load(ctx: &mut WindowCtx<'_>) -> Self {
		let mut root = ctx.root();
		let width = root.bounds().size.width;

		// raw: a handle validates the resource once and holds its length, then
		// streams it SIZE bytes at a time (the last chunk is short).
		let handle =
			raw::ResourceHandle::load(resources::DATA_RAW).pbl_expect(c"DATA_RAW declared");
		let mut bytes = Vec::with_capacity(handle.len());
		for chunk in handle.iter::<8>() {
			bytes.extend_from_slice(&chunk);
		}
		// the whole thing in one call, for when it fits in the app heap
		let _whole = raw::load_all(resources::DATA_RAW, PebbleAlloc).pbl_expect(c"DATA_RAW loaded");
		// DATA_RAW is text, a NUL, then some trailing bytes: keep the text and its NUL. flint gets
		// data~flint.bin, aplite data~bw.bin, the rest data.bin, so the title differs per watch.
		let nul = bytes
			.iter()
			.position(|&b| b == 0)
			.pbl_expect(c"DATA_RAW has a NUL");
		bytes.truncate(nul + 1);
		let text = CString::from_vec_with_nul(bytes).pbl_expect(c"one NUL, at the end");

		// font: a rasterised .pbf. The title owns it below, and unloads it when it drops.
		let font = Font::load(resources::FONT_ROBOTO_18);
		// a pre-baked `pbf` resource loads identically; nothing keeps this one, so it unloads at
		// the end of `load`
		let _from_pbf = Font::load(resources::FONT_PBF);

		// A round screen clips its corners; `pbl_round` is one of the cfg flags
		// the build script sets from the platform's tags
		let inset = if cfg!(pbl_round) { 24 } else { 0 };
		let mut title = TextLayer::new(Rectangle::new(
			Point::new(inset as i32, 0),
			Size::new(width - 2 * inset, 38),
		))
		.pbl_expect(c"title layer");
		title
			.set_font(font)
			.set_text(text)
			.set_text_colour(Colour { argb: 0xF5 });
		root.add_child(&mut title);

		// png / bitmap / pbi all become a GBitMap via the same loader. IMAGE_PNG
		// has an image~<platform>.png for both watches and IMAGE_PBI a
		// logo~<platform>.pbi, so those two layers show a different asset on each;
		// IMAGE_BITMAP compiles the one shared logo.png. Dropping a GBitMap
		// destroys the OS-side bitmap: `_from_bitmap` frees itself at the end of
		// `load`, while the layers own the two they show.
		let _from_bitmap =
			GBitMap::load(resources::IMAGE_BITMAP).pbl_expect(c"IMAGE_BITMAP declared");
		let mut image = BitmapLayer::new(Rectangle::new(Point::new(0, 40), Size::new(width, 56)))
			.pbl_expect(c"image layer");
		image.set_bitmap(GBitMap::load(resources::IMAGE_PNG).pbl_expect(c"IMAGE_PNG declared"));
		root.add_child(&mut image);

		let mut pbi = BitmapLayer::new(Rectangle::new(Point::new(0, 98), Size::new(width, 28)))
			.pbl_expect(c"pbi layer");
		pbi.set_bitmap(GBitMap::load(resources::IMAGE_PBI).pbl_expect(c"IMAGE_PBI declared"));
		root.add_child(&mut pbi);

		// svg: one file compiles to a draw command image. The firmware strokes and
		// fills it from the resource at draw time, so it scales with no palette
		// and no decode buffer. The layer owns the image and draws it itself.
		let ferris = DrawCommand::load(resources::FERRIS_PDC).pbl_expect(c"FERRIS_PDC declared");
		let ferris_size = ferris.bounds();
		let mut ferris_layer = DataLayer::new_with_data(
			Rectangle::new(
				Point::new(((width - ferris_size.width) / 2) as i32, 126),
				ferris_size,
			),
			ferris,
		)
		.pbl_expect(c"ferris layer");
		ferris_layer.set_draw(draw_ferris);
		root.add_child(&mut ferris_layer);

		// svg: a directory of frames compiles to a draw command sequence. The
		// layer draws the frame for the elapsed time, and a repeating timer
		// advances that time.
		let sequence =
			DrawCommandSequence::load(resources::SPINNER_PDCS).pbl_expect(c"SPINNER_PDCS declared");
		let spinner_size = sequence.bounds();
		let mut spinner = DataLayer::new_with_data(
			Rectangle::new(
				Point::new((width - spinner_size.width - 2) as i32, 4),
				spinner_size,
			),
			sequence,
		)
		.pbl_expect(c"spinner layer");
		spinner.set_draw(draw_spinner);
		root.add_child(&mut spinner);
		let spinner_timer = Timer::repeating(SPINNER_STEP_MS, advance_spinner);

		// a prebaked .pdc loads identically, image or sequence; nothing keeps this
		// one, so it drops (and is destroyed) at the end of `load`
		let _prebaked = DrawCommandSequence::load(resources::SPINNER_PREBAKED)
			.pbl_expect(c"SPINNER_PREBAKED declared");

		// vibe: the score compiles and gets a handle, but the current SDK has no
		// public "play this score" call; apps use the in-memory vibes_* API
		// (vibes_short_pulse, vibes_enqueue_custom_pattern).
		let _buzz = unsafe { sys::resource_get_handle(resources::BUZZ.0.get()) };

		// icon needs no code: it is wired into the app header as the menu icon at
		// packaging time. (`resources::APP_ICON` still exists if you want it.)

		Self {
			_title: title,
			_image: image,
			_pbi: pbi,
			_ferris: ferris_layer,
			spinner,
			_spinner_timer: spinner_timer,
		}
	}
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	ROOT.set(Window::new(None).pbl_expect(c"window created"));
	UiQueue::new(ui).push(&ROOT, false);
}

fn draw_ferris(image: &DrawCommand, _bounds: Rectangle, ctx: &mut GContext<'_>) {
	ctx.draw_pdc(image, Point::zero());
}

fn draw_spinner(sequence: &DrawCommandSequence, _bounds: Rectangle, ctx: &mut GContext<'_>) {
	ctx.draw_pdc_sequence(sequence, SPINNER_MS.get(), Point::zero());
}

/// Step the spinner on and redraw it, then run again after the next step. The sequence's play count
/// is `0xFFFF`, so the firmware wraps the elapsed time for us.
fn advance_spinner() -> Option<u32> {
	SPINNER_MS.set(SPINNER_MS.get().wrapping_add(SPINNER_STEP_MS));
	ROOT.with(&mut UiGuard::enter(), |layout, _| {
		if let Some(layout) = layout {
			layout.spinner.mark_dirty();
		}
	});
	Some(SPINNER_STEP_MS)
}
