//! A safe Rust SDK for Pebble watchapps and watchfaces, built on [`gneiss_sys`].
//!
//! Applications on PebbleOS do not have "bare-metal" hardware control, the OS owns the hardware and
//! the main event loop. The firmware loads and calls into your application binary at launch, where
//! you build your first window and register handlers, then return, yielding control back to the OS
//! event loop.
//!
//! # Getting started
//!
//! `cargo gneiss new` generates an app, `cargo gneiss build` builds its `.pbw` bundle, and
//! `cargo gneiss install` puts it on a watch. Every app has one
//! [`#[gneiss::entry]`](macro@entry) function:
//!
//! ```ignore
//! #![no_std]
//! #![no_main]
//! use gneiss::ui::{Nav, StaticWindow, UiGuard, Window};
//!
//! static HOME: StaticWindow<Home, ()> = StaticWindow::new();
//!
//! #[gneiss::entry]
//! fn main(ui: &mut UiGuard) {
//!     HOME.set(Window::new(None).expect("home window"));
//!     Nav::new(ui).push(&HOME, false);
//! }
//! ```
//!
//! `main` builds the first window and returns; the macro then subscribes your services and hands
//! over to the event loop. An app with no window on the stack exits.
//!
//! # Platforms
//!
//! Each watch model is a platform: `emery` (Pebble Time 2), `basalt`, `chalk`, `diorite`,
//! `flint`, `gabbro` and `aplite`. An app declares the ones it supports as features forwarding to
//! gneiss, e.g. `emery = ["gneiss/emery"]`, and `cargo gneiss build` builds it once for each. Code
//! can check the current platform's traits with cfgs the build sets, such as `cfg(pbl_round)` and
//! `cfg(pbl_touch)`.
//!
//! # Features
//!
//! - One per platform, as above. Exactly one is enabled in each build.
//! - `debug-logs` (default): [`debug!`](log::debug) and [`verbose!`](log::verbose) log. Without it
//!   they compile to nothing, gneiss's own included.
//!
//! # Concurrency & Cross-Task Sync
//!
//! All app code runs on a single task, so there's no concurrency to guard against, only reentrancy
//! (see [`UiGuard`](ui::UiGuard)).
//! Read-modify-write atomics never complete on the Pebble Time 2, so the app target leaves them
//! out entirely: there is no `Arc`, and no `fetch_add` or `compare_exchange`; Where you would reach
//! for Arc/Mutex, instead use the single-threaded versions behind `unsafe impl Sync` wrappers.
//!
//! # Panics
//!
//! A panic logs its message and location to the app log, and the firmware closes the app.
//! Building with `-Cpanic=immediate-abort` drops the panic machinery, a few KiB, at the cost of
//! silent panics; it also makes `gneiss::ufmt` available, for formatting without `core::fmt`.
#![no_std]
#![doc(test(attr(no_std, no_main)))]
#![deny(
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::string_slice,
	clippy::unchecked_time_subtraction
)]
#![warn(clippy::unreachable, clippy::unimplemented, clippy::todo)]
#![feature(
	const_array,
	const_trait_impl,
	ptr_cast_slice,
	auto_traits,
	negative_impls,
	associated_type_defaults
)]

pub use gneiss_macros::{entry, service};
pub use gneiss_sys as sys;
pub use linkme;
/// Present when building with `-Cpanic=immediate-abort`. `use gneiss::ufmt;` puts it in scope for
/// `ufmt`'s macros.
#[cfg(panic = "immediate-abort")]
pub use ufmt;

pub mod alloc;

pub mod log;
pub mod service;
pub mod ui;

pub mod link;

// Typed Resources
pub mod resource;

// Strong type wrappers for sys types
mod types;
pub use types::{accel, battery, compass, connection, dict, focus, health, time, touch, wakeup};

#[cfg(feature = "__doctest")]
#[panic_handler]
fn __doctest_panic(info: &core::panic::PanicInfo) -> ! {
	gneiss_sys::panic::panic_fmt(info)
}

#[cfg(feature = "__doctest")]
#[global_allocator]
static __DOCTEST_ALLOC: alloc::PebbleAlloc = alloc::PebbleAlloc;

#[cfg(feature = "__doctest")]
#[doc(hidden)]
#[macro_export]
macro_rules! __doctest_slices {
	() => {
		#[::gneiss::linkme::distributed_slice]
		#[linkme(crate = ::gneiss::linkme)]
		static GNEISS_SERVICES: [fn()];
		#[::gneiss::linkme::distributed_slice]
		#[linkme(crate = ::gneiss::linkme)]
		static GNEISS_LAUNCH_HOOKS: [fn()];
	};
}
