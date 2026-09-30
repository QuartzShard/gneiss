//! Declares every currently-supported resource kind (see Cargo.toml) and shows
//! how each is loaded at runtime. The generated `resources` module gives one
//! typed key per declared resource, named after the declaration: the key's type
//! follows the declared `type`.
#![no_std]
#![no_main]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::mem::forget;

use gneiss::{
	resource::{
		image::{DrawCommand, DrawCommandSequence, GBitMap},
		raw,
	},
	sys,
};

#[gneiss::entry]
fn main() {
	let window = unsafe { sys::window_create() };
	let root = unsafe { sys::window_get_root_layer(window) };
	let bounds = unsafe { sys::layer_get_bounds(root) };

	// raw: a handle validates the resource once and holds its length, then
	// streams it SIZE bytes at a time (the last chunk is short). The text
	// layer below keeps the pointer, so the buffer has to outlive this
	// function: leak it.
	let handle = raw::ResourceHandle::load(resources::DATA_RAW).expect("DATA_RAW declared");
	let mut bytes = Vec::with_capacity(handle.len());
	for chunk in handle.iter::<8>() {
		bytes.extend_from_slice(&chunk);
	}
	let raw_text = Box::leak(bytes.into_boxed_slice());
	// the whole thing in one call, for when it fits in the app heap
	let _whole = raw::load_all(resources::DATA_RAW).expect("DATA_RAW loaded");

	// font: a rasterised .pbf, still loaded through `sys` (no safe wrapper yet).
	// A pre-baked `pbf` resource loads identically.
	let font = unsafe {
		sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_ROBOTO_18.0.get()))
	};
	let _font_from_pbf = unsafe {
		sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_PBF.0.get()))
	};
	// A round screen clips its corners; `pbl_round` is one of the cfg flags
	// the build script sets from the platform's tags
	let inset = if cfg!(pbl_round) { 24 } else { 0 };
	unsafe {
		let text = sys::text_layer_create(sys::GRect {
			origin: sys::GPoint { x: inset, y: 0 },
			size:   sys::GSize {
				w: bounds.size.w - 2 * inset,
				h: 38,
			},
		});
		sys::text_layer_set_font(text, font);
		// DATA_RAW is NUL-terminated text: flint gets data~flint.bin, emery has no
		// variant and falls back to data.bin, so the title differs per watch.
		sys::text_layer_set_text(text, raw_text.as_ptr().cast());
		sys::text_layer_set_text_color(text, sys::GColor8 { argb: 0xF5 });
		sys::layer_add_child(root, sys::text_layer_get_layer(text));
	}

	// png / bitmap / pbi all become a GBitMap via the same safe loader. IMAGE_PNG
	// has an image~<platform>.png for both watches and IMAGE_PBI a
	// logo~<platform>.pbi, so those two layers show a different asset on each;
	// IMAGE_BITMAP compiles the one shared logo.png. Dropping a GBitMap destroys
	// the OS-side bitmap: `_from_bitmap` frees itself at the end of `main`, while
	// the two the layers keep are forgotten below instead.
	let bitmap = GBitMap::load(resources::IMAGE_PNG).expect("IMAGE_PNG declared");
	let _from_bitmap = GBitMap::load(resources::IMAGE_BITMAP).expect("IMAGE_BITMAP declared");
	let from_pbi = GBitMap::load(resources::IMAGE_PBI).expect("IMAGE_PBI declared");
	unsafe {
		let image = sys::bitmap_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 40 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 56,
			},
		});
		sys::bitmap_layer_set_bitmap(image, bitmap.as_ptr());
		sys::layer_add_child(root, sys::bitmap_layer_get_layer(image));

		let pbi_image = sys::bitmap_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 98 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 28,
			},
		});
		sys::bitmap_layer_set_bitmap(pbi_image, from_pbi.as_ptr());
		sys::layer_add_child(root, sys::bitmap_layer_get_layer(pbi_image));
	}
	// the layers keep the raw pointers, so neither wrapper may ever drop
	forget(bitmap);
	forget(from_pbi);

	// svg: one file compiles to a draw command image. The firmware strokes and
	// fills it from the resource at draw time, so it scales with no palette
	// and no decode buffer; there is still nothing that draws it for us.
	let ferris = DrawCommand::load(resources::FERRIS_PDC).expect("FERRIS_PDC declared");
	unsafe {
		let ferris_size = sys::gdraw_command_image_get_bounds_size(ferris.as_ptr());
		let ferris_layer = sys::layer_create_with_data(
			sys::GRect {
				origin: sys::GPoint {
					x: (bounds.size.w - ferris_size.w) / 2,
					y: 126,
				},
				size:   ferris_size,
			},
			size_of::<*mut sys::GDrawCommandImage>(),
		);
		sys::layer_get_data(ferris_layer)
			.cast::<*mut sys::GDrawCommandImage>()
			.write(ferris.as_ptr());
		sys::layer_set_update_proc(ferris_layer, Some(draw_ferris));
		sys::layer_add_child(root, ferris_layer);
	}
	forget(ferris);

	// svg: a directory of frames compiles to a draw command sequence. Nothing
	// draws this one for us either: a layer's update_proc asks the sequence for
	// the frame at an elapsed time, and a repeating app timer advances that
	// time. The state rides in the layer's own callback data rather than a
	// static.
	let sequence =
		DrawCommandSequence::load(resources::SPINNER_PDCS).expect("SPINNER_PDCS declared");
	unsafe {
		let spinner = sys::layer_create_with_data(
			sys::GRect {
				origin: sys::GPoint {
					x: bounds.size.w - SPINNER_SIZE - 2,
					y: 4,
				},
				size:   sys::GSize {
					w: SPINNER_SIZE,
					h: SPINNER_SIZE,
				},
			},
			siz_of::<Spinner>(),
		);
		sys::layer_get_data(spinner)
			.cast::<Spinner>()
			.write(Spinner {
				sequence:   sequence.as_ptr(),
				elapsed_ms: 0,
			});
		sys::layer_set_update_proc(spinner, Some(draw_spinner));
		sys::layer_add_child(root, spinner);
		sys::app_timer_register(SPINNER_STEP_MS, Some(advance_spinner), spinner.cast());
	}
	forget(sequence);
	// a prebaked .pdc loads identically, image or sequence; nothing keeps this
	// one, so it drops (and is destroyed) at the end of `main`
	let _prebaked =
		DrawCommandSequence::load(resources::SPINNER_PREBAKED).expect("SPINNER_PREBAKED declared");

	// vibe: the score compiles and gets a handle, but the current SDK has no
	// public "play this score" call; apps use the in-memory vibes_* API
	// (vibes_short_pulse, vibes_enqueue_custom_pattern).
	let _buzz = unsafe { sys::resource_get_handle(resources::BUZZ.0.get()) };

	// icon needs no code: it is wired into the app header as the menu icon at
	// packaging time. (`resources::APP_ICON` still exists if you want it.)

	unsafe { sys::window_stack_push(window, false) };
}

/// Matches the `spinner` resource's `frameDurationMs`.
const SPINNER_STEP_MS: u32 = 60;
const SPINNER_SIZE: i16 = 28;

struct Spinner {
	sequence:   *mut sys::GDrawCommandSequence,
	elapsed_ms: u32,
}

/// The OS calls an update_proc when its layer needs to redraw, handing over a
/// context to draw into.
unsafe extern "C" fn draw_ferris(layer: *mut sys::Layer, ctx: *mut sys::GContext) {
	unsafe {
		let image = *sys::layer_get_data(layer).cast::<*mut sys::GDrawCommandImage>();
		sys::gdraw_command_image_draw(ctx, image, sys::GPoint { x: 0, y: 0 });
	}
}

/// Same, for the animated sequence: pick the frame for the elapsed time.
unsafe extern "C" fn draw_spinner(layer: *mut sys::Layer, ctx: *mut sys::GContext) {
	unsafe {
		let spinner = &*sys::layer_get_data(layer).cast::<Spinner>();
		let frame =
			sys::gdraw_command_sequence_get_frame_by_elapsed(spinner.sequence, spinner.elapsed_ms);
		if !frame.is_null() {
			sys::gdraw_command_frame_draw(ctx, spinner.sequence, frame, sys::GPoint { x: 0, y: 0 });
		}
	}
}

/// An app timer fires once, so re-register to keep going. The sequence's play
/// count is `0xFFFF`, so the firmware wraps the elapsed time for us.
unsafe extern "C" fn advance_spinner(data: *mut core::ffi::c_void) {
	unsafe {
		let layer: *mut sys::Layer = data.cast();
		let spinner = &mut *sys::layer_get_data(layer).cast::<Spinner>();
		spinner.elapsed_ms = spinner.elapsed_ms.wrapping_add(SPINNER_STEP_MS);
		sys::layer_mark_dirty(layer);
		sys::app_timer_register(SPINNER_STEP_MS, Some(advance_spinner), data);
	}
}
