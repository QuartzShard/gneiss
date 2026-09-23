//! Declares every currently-supported resource kind (see Cargo.toml) and shows
//! how each is loaded at runtime. The `#[entry]` macro generates the `resources`
//! module, giving one `u32` const per declared resource.
#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec;

use gneiss::sys;

#[gneiss::entry]
fn main() {
	unsafe {
		let window = sys::window_create();
		let root = sys::window_get_root_layer(window);
		let bounds = sys::layer_get_bounds(root);

		// raw: read the bytes back into a heap buffer. The text layer below keeps
		// the pointer, so the buffer has to outlive this function — leak it.
		let raw = sys::resource_get_handle(resources::DATA_RAW);
		let size = sys::resource_size(raw);
		let mut buf = vec![0u8; size];
		let _read = sys::resource_load(raw, buf.as_mut_ptr(), size);
		let raw_text = alloc::boxed::Box::leak(buf.into_boxed_slice());

		// font: a rasterised .pbf. A pre-baked `pbf` resource loads identically.
		let font = sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_ROBOTO_18));
		let _font_from_pbf =
			sys::fonts_load_custom_font(sys::resource_get_handle(resources::FONT_PBF));
		let text = sys::text_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 0 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 40,
			},
		});
		sys::text_layer_set_font(text, font);
		// DATA_RAW is NUL-terminated text: flint gets data~flint.bin, emery has no
		// variant and falls back to data.bin, so the title differs per watch.
		sys::text_layer_set_text(text, raw_text.as_ptr().cast());
		sys::layer_add_child(root, sys::text_layer_get_layer(text));

		// png / bitmap / pbi all become a GBitmap via the same call. IMAGE_PNG has
		// an image~<platform>.png for both watches and IMAGE_PBI a logo~<platform>
		// .pbi, so those two layers show a different asset on each; IMAGE_BITMAP
		// compiles the one shared logo.png.
		let bitmap = sys::gbitmap_create_with_resource(resources::IMAGE_PNG);
		let _from_bitmap = sys::gbitmap_create_with_resource(resources::IMAGE_BITMAP);
		let from_pbi = sys::gbitmap_create_with_resource(resources::IMAGE_PBI);
		let image = sys::bitmap_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 50 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 60,
			},
		});
		sys::bitmap_layer_set_bitmap(image, bitmap);
		sys::layer_add_child(root, sys::bitmap_layer_get_layer(image));

		let pbi_image = sys::bitmap_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 115 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 45,
			},
		});
		sys::bitmap_layer_set_bitmap(pbi_image, from_pbi);
		sys::layer_add_child(root, sys::bitmap_layer_get_layer(pbi_image));

		// vibe: the score compiles and gets a handle, but the current SDK has no
		// public "play this score" call; apps use the in-memory vibes_* API
		// (vibes_short_pulse, vibes_enqueue_custom_pattern).
		let _buzz = sys::resource_get_handle(resources::BUZZ);

		// icon needs no code: it is wired into the app header as the menu icon at
		// packaging time. (`resources::APP_ICON` still exists if you want it.)

		sys::window_stack_push(window, false);
	}
}
