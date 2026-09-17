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
		sys::text_layer_set_text(text, c"Resources".as_ptr());
		sys::layer_add_child(root, sys::text_layer_get_layer(text));

		// png / bitmap / pbi all become a GBitmap via the same call.
		let bitmap = sys::gbitmap_create_with_resource(resources::IMAGE_PNG);
		let _from_bitmap = sys::gbitmap_create_with_resource(resources::IMAGE_BITMAP);
		let _from_pbi = sys::gbitmap_create_with_resource(resources::IMAGE_PBI);
		let image = sys::bitmap_layer_create(sys::GRect {
			origin: sys::GPoint { x: 0, y: 50 },
			size:   sys::GSize {
				w: bounds.size.w,
				h: 60,
			},
		});
		sys::bitmap_layer_set_bitmap(image, bitmap);
		sys::layer_add_child(root, sys::bitmap_layer_get_layer(image));

		// raw: read the bytes back into a heap buffer.
		let raw = sys::resource_get_handle(resources::DATA_RAW);
		let size = sys::resource_size(raw);
		let mut buf = vec![0u8; size];
		let _read = sys::resource_load(raw, buf.as_mut_ptr(), size);

		// vibe: the score compiles and gets a handle, but the current SDK has no
		// public "play this score" call; apps use the in-memory vibes_* API
		// (vibes_short_pulse, vibes_enqueue_custom_pattern).
		let _buzz = sys::resource_get_handle(resources::BUZZ);

		// icon needs no code: it is wired into the app header as the menu icon at
		// packaging time. (`resources::APP_ICON` still exists if you want it.)

		sys::window_stack_push(window, false);
	}
}
