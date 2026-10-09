//! Declares every currently-supported resource kind (see Cargo.toml) and shows
//! how each is loaded at runtime. The generated `resources` module gives one
//! typed key per declared resource, named after the declaration: the key's type
//! follows the declared `type`.
#![no_std]
#![no_main]
extern crate alloc;

use alloc::{ffi::CString, vec::Vec};
use core::{
	ffi::c_void,
	ptr,
	sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use gneiss::{
	alloc::PebbleAlloc,
	resource::{
		image::{DrawCommand, DrawCommandSequence, GBitMap},
		raw,
	},
	sys,
	ui::{
		AsLayer, BitmapLayer, DataLayer, Layer, Layout, Nav, StaticWindow, TextLayer, UiGuard,
		Window, WindowCtx,
		graphics::{Colour, GContext, Point, Rectangle, Size},
	},
};

static ROOT: StaticWindow<Resources, ()> = StaticWindow::new();

/// Matches the `spinner` resource's `frameDurationMs`.
const SPINNER_STEP_MS: u32 = 60;
const SPINNER_SIZE: u32 = 28;

// Elapsed time into the spinner's sequence, advanced by a raw app timer until gneiss wraps timers.
static SPINNER_MS: AtomicU32 = AtomicU32::new(0);
static SPINNER_TIMER: AtomicPtr<sys::AppTimer> = AtomicPtr::new(ptr::null_mut());

/// Everything on screen, created when the window loads and dropped when it unloads. Each layer owns
/// the resource it shows, so nothing needs leaking.
struct Resources {
	title:    TextLayer<CString>,
	_image:   BitmapLayer,
	_pbi:     BitmapLayer,
	_ferris:  DataLayer<DrawCommand>,
	_spinner: DataLayer<DrawCommandSequence>,
	font:     sys::GFont,
}

impl Layout for Resources {
	fn load(ctx: &mut WindowCtx<'_, Self>) -> Self {
		let mut root = ctx.root();
		let width = root.bounds().size.width;

		// raw: a handle validates the resource once and holds its length, then
		// streams it SIZE bytes at a time (the last chunk is short).
		let handle = raw::ResourceHandle::load(resources::DATA_RAW).expect("DATA_RAW declared");
		let mut bytes = Vec::with_capacity(handle.len());
		for chunk in handle.iter::<8>() {
			bytes.extend_from_slice(&chunk);
		}
		// the whole thing in one call, for when it fits in the app heap
		let _whole = raw::load_all(resources::DATA_RAW, PebbleAlloc).expect("DATA_RAW loaded");
		// DATA_RAW is text, a NUL, then some trailing bytes: keep the text and its NUL. flint gets
		// data~flint.bin, aplite data~bw.bin, the rest data.bin, so the title differs per watch.
		let nul = bytes
			.iter()
			.position(|&b| b == 0)
			.expect("DATA_RAW has a NUL");
		bytes.truncate(nul + 1);
		let text = CString::from_vec_with_nul(bytes).expect("one NUL, at the end");

		// font: a rasterised .pbf, still loaded through `sys` (no safe wrapper yet).
		// A pre-baked `pbf` resource loads identically.
		let font = unsafe {
			sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_ROBOTO_18.0.get()))
		};
		unsafe {
			let font_from_pbf =
				sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_PBF.0.get()));
			sys::fonts_unload_custom_font(font_from_pbf);
		}

		// A round screen clips its corners; `pbl_round` is one of the cfg flags
		// the build script sets from the platform's tags
		let inset = if cfg!(pbl_round) { 24 } else { 0 };
		let mut title = TextLayer::new(Rectangle::new(
			Point::new(inset as i32, 0),
			Size::new(width - 2 * inset, 38),
		))
		.expect("title layer");
		// SAFETY: the font stays loaded until unload, after the title is dropped
		unsafe { title.set_font(font) };
		title.set_text(text);
		title.set_text_color(Colour { argb: 0xF5 });
		root.add_child(&mut title);

		// png / bitmap / pbi all become a GBitMap via the same loader. IMAGE_PNG
		// has an image~<platform>.png for both watches and IMAGE_PBI a
		// logo~<platform>.pbi, so those two layers show a different asset on each;
		// IMAGE_BITMAP compiles the one shared logo.png. Dropping a GBitMap
		// destroys the OS-side bitmap: `_from_bitmap` frees itself at the end of
		// `load`, while the layers own the two they show.
		let _from_bitmap = GBitMap::load(resources::IMAGE_BITMAP).expect("IMAGE_BITMAP declared");
		let mut image = BitmapLayer::new(Rectangle::new(Point::new(0, 40), Size::new(width, 56)))
			.expect("image layer");
		image.set_bitmap(GBitMap::load(resources::IMAGE_PNG).expect("IMAGE_PNG declared"));
		root.add_child(&mut image);

		let mut pbi = BitmapLayer::new(Rectangle::new(Point::new(0, 98), Size::new(width, 28)))
			.expect("pbi layer");
		pbi.set_bitmap(GBitMap::load(resources::IMAGE_PBI).expect("IMAGE_PBI declared"));
		root.add_child(&mut pbi);

		// svg: one file compiles to a draw command image. The firmware strokes and
		// fills it from the resource at draw time, so it scales with no palette
		// and no decode buffer. The layer owns the image and draws it itself.
		let ferris = DrawCommand::load(resources::FERRIS_PDC).expect("FERRIS_PDC declared");
		let ferris_size = unsafe { sys::gdraw_command_image_get_bounds_size(ferris.as_ptr()) };
		let ferris_size = Size::new(ferris_size.w as u32, ferris_size.h as u32);
		let mut ferris_layer = DataLayer::new_with_data(
			Rectangle::new(
				Point::new(((width - ferris_size.width) / 2) as i32, 126),
				ferris_size,
			),
			ferris,
		)
		.expect("ferris layer");
		ferris_layer.set_draw(draw_ferris);
		root.add_child(&mut ferris_layer);

		// svg: a directory of frames compiles to a draw command sequence. The
		// layer draws the frame for the elapsed time, and a repeating app timer
		// advances that time.
		let sequence =
			DrawCommandSequence::load(resources::SPINNER_PDCS).expect("SPINNER_PDCS declared");
		let mut spinner = DataLayer::new_with_data(
			Rectangle::new(
				Point::new((width - SPINNER_SIZE - 2) as i32, 4),
				Size::new(SPINNER_SIZE, SPINNER_SIZE),
			),
			sequence,
		)
		.expect("spinner layer");
		spinner.set_draw(draw_spinner);
		root.add_child(&mut spinner);
		let timer = unsafe {
			sys::app_timer_register(
				SPINNER_STEP_MS,
				Some(advance_spinner),
				spinner.as_ptr().cast(),
			)
		};
		SPINNER_TIMER.store(timer, Ordering::Relaxed);

		// a prebaked .pdc loads identically, image or sequence; nothing keeps this
		// one, so it drops (and is destroyed) at the end of `load`
		let _prebaked = DrawCommandSequence::load(resources::SPINNER_PREBAKED)
			.expect("SPINNER_PREBAKED declared");

		// vibe: the score compiles and gets a handle, but the current SDK has no
		// public "play this score" call; apps use the in-memory vibes_* API
		// (vibes_short_pulse, vibes_enqueue_custom_pattern).
		let _buzz = unsafe { sys::resource_get_handle(resources::BUZZ.0.get()) };

		// icon needs no code: it is wired into the app header as the menu icon at
		// packaging time. (`resources::APP_ICON` still exists if you want it.)

		Self {
			title,
			_image: image,
			_pbi: pbi,
			_ferris: ferris_layer,
			_spinner: spinner,
			font,
		}
	}

	fn unload(self, _ctx: &mut WindowCtx<'_, Self>) {
		// The timer holds the spinner layer's pointer, so it stops before the layer goes
		let timer = SPINNER_TIMER.load(Ordering::Relaxed);
		SPINNER_TIMER.store(ptr::null_mut(), Ordering::Relaxed);
		unsafe { sys::app_timer_cancel(timer) };
		// The title points at the font, so it goes first
		let Self { title, font, .. } = self;
		drop(title);
		unsafe { sys::fonts_unload_custom_font(font) };
	}
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) {
	ROOT.set(Window::new(None).expect("window created"));
	Nav::new(ui).push(&ROOT, false);
}

fn draw_ferris(image: &DrawCommand, _bounds: Rectangle, ctx: &mut GContext<'_>) {
	ctx.draw_pdc(image, Point::zero());
}

fn draw_spinner(sequence: &DrawCommandSequence, _bounds: Rectangle, ctx: &mut GContext<'_>) {
	ctx.draw_pdc_sequence(sequence, SPINNER_MS.load(Ordering::Relaxed), Point::zero());
}

/// An app timer fires once, so re-register to keep going. The sequence's play
/// count is `0xFFFF`, so the firmware wraps the elapsed time for us.
unsafe extern "C" fn advance_spinner(layer: *mut c_void) {
	let elapsed = SPINNER_MS.load(Ordering::Relaxed);
	SPINNER_MS.store(elapsed.wrapping_add(SPINNER_STEP_MS), Ordering::Relaxed);
	unsafe {
		sys::layer_mark_dirty(layer.cast());
		let timer = sys::app_timer_register(SPINNER_STEP_MS, Some(advance_spinner), layer);
		SPINNER_TIMER.store(timer, Ordering::Relaxed);
	}
}
