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
//! use gneiss::ui::{UiQueue, StaticWindow, UiGuard, Window};
//!
//! static HOME: StaticWindow<Home> = StaticWindow::new();
//!
//! #[gneiss::entry]
//! fn main(ui: &mut UiGuard) {
//!     HOME.set(Window::new(None).expect("home window"));
//!     UiQueue::new(ui).push(&HOME, false);
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
//! - `logs`: [`log`](mod@log)'s macros log. Without it they compile to nothing, gneiss's own
//!   included; panics still log.
//! - `debug-logs` (default, implies `logs`): [`debug!`](log::debug) and [`verbose!`](log::verbose)
//!   log. Without it they compile to nothing, gneiss's own included.
//! - `trace-ui` (implies `logs`): every window logs its `load`, `appear`, `disappear` and `unload`, the drop of its
//!   handle and its destruction, as `ui <event> <window> <layout type>`, for tests to follow the UI
//!   from the log. Without it, none of this is compiled in.
//! - `user-panic-handler`: gneiss defines no `#[panic_handler]` and doesn't require
//!   `-Cpanic=immediate-abort`, for an app that brings its own.
//!
//! # Concurrency & Cross-Task Sync
//!
//! All app code runs on a single task, so there's no concurrency to guard against, only reentrancy
//! (see [`UiGuard`](ui::UiGuard)).
//! Read-modify-write atomics never complete on the Pebble Time 2, so the app target leaves them
//! out entirely: there is no `Arc`, and no `fetch_add` or `compare_exchange`; Where you would reach
//! for Arc/Mutex, use `Rc` and [`SyncCell`].
//!
//! # Panics
//!
//! Apps build with `-Zunstable-options -Cpanic=immediate-abort`, which drops the panic machinery
//! and `core::fmt` with it, a few KiB: a panic closes the app and prints no diagnostics. The
//! `user-panic-handler` feature lifts that requirement for an app that brings its own
//! `#[panic_handler]`, such as one that formats the message.
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
	associated_type_defaults,
	unboxed_closures,
	tuple_trait
)]

/// For [`AsBytes`](resource::persist::AsBytes) and [`make_persistible`]. Derive its traits through
/// this re-export with `#[bytemuck(crate = "gneiss::bytemuck")]`.
pub use bytemuck;
#[doc(hidden)]
pub use gneiss_macros::__pbl_log;
pub use gneiss_macros::{entry, make_persistible, menu, pbl_fmt_write, persist_keys, service};
// gneiss's own macro output names `::gneiss`, as an app's does
extern crate self as gneiss;
pub use gneiss_sys as sys;
pub use linkme;

pub mod alloc;
pub mod fail;
pub mod fmt;

/// What most apps use everywhere: `use gneiss::prelude::*;`
pub mod prelude {
	pub use crate::{fail::PblUnwrap, pbl_assert, pbl_fmt_alloc, pbl_fmt_write, pbl_panic};
}

pub mod light;
pub mod log;
pub mod service;
pub mod timer;
pub mod ui;
pub mod vibes;

pub mod link;

// Typed Resources
pub mod resource;

// Strong type wrappers for sys types
mod types;
pub use types::*;

#[cfg(not(feature = "user-panic-handler"))]
#[panic_handler]
fn __gneiss_panic(info: &core::panic::PanicInfo) -> ! {
	gneiss_sys::panic::panic_halt(info)
}

#[cfg(all(not(panic = "immediate-abort"), not(feature = "user-panic-handler")))]
compile_error!(
	"gneiss apps build with `-Zunstable-options -Cpanic=immediate-abort` in their rustflags, or \
	 enable gneiss's `user-panic-handler` feature and bring a panic handler"
);

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
