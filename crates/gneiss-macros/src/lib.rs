//! The `#[entry]` and `#[service]` attribute macros, used as `gneiss::entry` and `gneiss::service`.
#![allow(unused)]
#![allow(clippy::needless_pass_by_value)]
mod entry;
mod service;

use proc_macro::TokenStream;
#[proc_macro_attribute]
/// Takes a `fn main() {}` or `fn main(&mut UiGuard) {}` and wraps it in the required ceremony to be a valid entry for the watch,
/// as well as assigning a panic handler.
///
/// Your main fn MUST return or the `sys::app_event_loop` call after it in init is unreachable. You build
/// your UI and state here, you don't drive execution.
///
/// Takes the following optional arguments:
/// - panic: path to a `fn(&core::panic::PanicInfo) -> !`, defaults to `::gneiss::sys::panic::panic_fmt`
/// - `global_alloc`: path to a unit-struct `GlobalAlloc` to register as the global allocator,
///   defaults to `::gneiss::alloc::PebbleAlloc`.
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
	entry::entry(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
/// Subscribe a free function to a gneiss service, once `main` has returned.
///
/// ```ignore
/// #[gneiss::service(TickService, time::TimeUnits::MINUTES)]
/// fn on_minute(time: time::TickTime, _changed: time::TimeUnits) {}
/// ```
///
/// The first argument names the service; anything after it becomes the service's subscribe
/// argument, listed separated by commas and passed as a tuple when there are several. The function must have the service's handler
/// signature. Subscribing the same service twice in one app is a link error.
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
	service::service(attr.into(), item.into()).into()
}
