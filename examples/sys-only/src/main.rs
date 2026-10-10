#![no_std]
#![no_main]
//! Hello world over raw `gneiss-sys`, no `gneiss`: what `#[gneiss::entry]`
//! scaffolds, written out by hand. The panic handler, the exported `main` the
//! loader jumps to, and the closing `app_event_loop()` handoff. Service
//! registration belongs to `#[gneiss::service]`, and the generated `resources`
//! and `message_keys` modules name gneiss types, so neither is here.
//!
//! Also the bisection app for firmware updates: nothing but `gneiss-sys` in
//! the link, so when this app and a gneiss app disagree about surviving an
//! OTA, the break is on the gneiss side of the FFI surface (or vice versa).

use gneiss_sys::*;

/// See `gneiss_sys::panic` for options
#[panic_handler]
fn pbl_panic(info: &core::panic::PanicInfo<'_>) -> ! {
	gneiss_sys::panic::panic_fmt(info)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
	unsafe {
		let window = window_create();
		let root = window_get_root_layer(window);
		let bounds = layer_get_bounds(root);

		let text_layer = text_layer_create(GRect {
			origin: GPoint { x: 0, y: (bounds.size.h - 40) / 2 },
			size:   GSize { w: bounds.size.w, h: 40 },
		});
		text_layer_set_text(text_layer, c"Hello from raw gneiss-sys!".as_ptr());
		text_layer_set_text_alignment(text_layer, GTextAlignment_GTextAlignmentCenter);
		layer_add_child(root, text_layer_get_layer(text_layer));

		window_stack_push(window, false);
		app_event_loop();
	}
}
