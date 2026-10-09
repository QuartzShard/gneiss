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
/// - panic: path to a `fn(&core::panic::PanicInfo) -> !`, defaults to `::gneiss::sys::panic::panic_minimal`
/// - `global_alloc`: path to a unit-struct `GlobalAlloc` to register as the global allocator,
///   defaults to `::gneiss::alloc::PebbleAlloc`.
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
	entry::entry(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
/// Register a freestanding fn as a service handler
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
	service::service(attr.into(), item.into()).into()
}
