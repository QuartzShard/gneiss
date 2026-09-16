#![allow(unused)]
#![allow(clippy::needless_pass_by_value)]
mod entry;
mod service;

use proc_macro::TokenStream;
#[proc_macro_attribute]
/// Takes a `fn main() {}` and wraps it in the required ceremony to be a valid entry for the watch,
/// as well as assigning a panic handler.
///
/// Takes the following optional arguments:
/// - panic: path to a `fn(&core::panic::PanicInfo) -> !`, defaults to `::gneiss::sys::panic::panic_minimal`
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
	entry::entry(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
/// Register a freestanding fn as a service handler
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
	service::service(attr.into(), item.into()).into()
}
