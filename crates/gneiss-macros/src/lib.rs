//! The `#[entry]`, `#[service]`, `#[menu]` and `#[make_persistible]` attribute macros and
//! `persist_keys!`, used through `gneiss`.
#![allow(unused)]
#![feature(proc_macro_diagnostic)]
#![allow(clippy::needless_pass_by_value)]
mod entry;
mod fmt;
mod menu;
mod persist;
mod service;

use proc_macro::TokenStream;
use syn::Token;
#[proc_macro_attribute]
/// Takes a `fn main() {}` or `fn main(&mut UiGuard) {}` and wraps it in the required ceremony to be a valid entry for the watch.
///
/// Your main fn MUST return or the `sys::app_event_loop` call after it in init is unreachable. You build
/// your UI and state here, you don't drive execution.
///
/// Takes the following optional arguments:
/// - `global_alloc`: path to a unit-struct `GlobalAlloc` to register as the global allocator,
///   defaults to `::gneiss::alloc::PebbleAlloc`.
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
	entry::entry(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
/// Subscribe a free function to a gneiss service, once `main` has returned.
///
/// ```ignore
/// #[gneiss::service(TickService, enabled_at_boot = true, time::TimeUnits::MINUTES)]
/// fn on_minute(time: time::DateTime, _changed: time::TimeUnits) {}
/// ```
///
/// The first argument names the service. An optional `enabled_at_boot` flag may be specified,
/// defaulting to true. Anything after that becomes the service's subscribe arguments, listed
/// separated by commas and passed as a tuple when there are several. The function must have the
/// service's handler signature.
/// Subscribing the same service twice in one app is a link error.
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
	service::service(attr.into(), item.into()).into()
}

#[proc_macro]
/// `write!` through the firmware's `snprintf`, with no `core::fmt`: append `format!`-style text to
/// a `StackText` or `HeapText`, returning `Err` if it didn't fit.
///
/// ```ignore
/// let (lap, secs, ms) = (3, 5, 20);
/// let _ = pbl_fmt_write!(text, "Lap {}: {:02}.{:03}", lap, secs, ms);
/// // text ends "Lap 3: 05.020"
/// ```
///
/// See `gneiss::fmt` for what it formats, the `{:s}`/`{:d}` letters, and how it differs from
/// `write!`.
pub fn pbl_fmt_write(input: TokenStream) -> TokenStream {
	fmt::pbl_fmt_write(input.into()).into()
}

#[proc_macro]
#[doc(hidden)]
/// `level, file, line, "format", args...`: log through `app_log` with `pbl_fmt_write!`'s format
/// language. Called by `gneiss::log`'s macros.
pub fn __pbl_log(input: TokenStream) -> TokenStream {
	fmt::pbl_log(input.into()).into()
}

#[proc_macro]
/// Declare the app's persist keys, numbered in order from 0 so no two collide, each with the type it
/// stores. Once per app: a second invocation anywhere would share keys, so it's a link error. The
/// app's exported `__gneiss_persist_keys`, a `u32`, holds the first key after the fixed entries,
/// where a `[T; ..]` tail starts.
///
/// ```ignore
/// gneiss::persist_keys! {
///     /// One key holding an `i32`
///     LAUNCHES: i32,
///     /// 16 keys, `NOTES.get(0)` to `NOTES.get(15)`
///     NOTES: [Note; 16],
///     /// Every key after the rest, last only
///     LOG: [Entry; ..],
/// }
/// ```
///
/// A plain entry is a `PersistKey`, `[T; len]` a `PersistBlock` and `[T; ..]` a `PersistTail`. `[T]`
/// alone is one key holding a slice. Add new entries at the end: moving one reads another's stored
/// value under its key. See `gneiss::resource::persist` for a worked example.
pub fn persist_keys(input: TokenStream) -> TokenStream {
	persist::persist_keys(input.into()).into()
}

#[proc_macro_attribute]
/// Make a struct storable whole, in one persist key, as its bytes.
///
/// ```ignore
/// #[gneiss::make_persistible]
/// struct HighScores {
///     top: [u32; 5],
/// }
/// ```
///
/// Adds `#[repr(C)]` unless the struct has `repr(C)` or `repr(transparent)` already (any other
/// `repr` is an error), derives whichever of `Clone`, `Copy`, `bytemuck::Pod` and
/// `bytemuck::Zeroable` it doesn't already derive, and implements `Persist` and `Recover`.
///
/// The struct must be at most 256 bytes. `Recover::recover_or_default` also needs it to implement
/// `Default`. `Pod` rejects a struct with padding: order fields largest first, or add explicit
/// padding fields. Generic structs aren't supported.
pub fn make_persistible(attr: TokenStream, item: TokenStream) -> TokenStream {
	persist::make_persistible(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
/// Implement `Menu`: put it on the `impl Menu for` block.
///
/// ```ignore
/// #[gneiss::menu]
/// impl Menu for Settings {
///     type Data = SettingsData;
///     fn get_num_rows(data: &SettingsData, _section: u16) -> u16 { data.len() }
///     fn draw_row(data: &SettingsData, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {}
/// }
/// ```
///
/// Each method left out is left unset for the firmware, which then uses its own default, rather
/// than calling the trait's default body. `get_header_height` and `draw_header` must be written
/// together, as must `get_separator_height` and `draw_separator`.
pub fn menu(attr: TokenStream, item: TokenStream) -> TokenStream {
	menu::menu(attr.into(), item.into()).into()
}

/// Parse an `, ident = true` shaped input, returning the value of the bool
fn parse_key(
	input: syn::parse::ParseStream<'_>,
	ident: &str,
	default: bool,
) -> Result<bool, syn::Error> {
	Ok(
		if input.peek(Token![,]) && input.peek2(syn::Ident) && input.peek3(Token![=]) {
			input.parse::<Token![,]>()?;
			let key = input.parse::<syn::Ident>()?;
			if key != ident {
				return Err(syn::Error::new(key.span(), format!("expected `{ident}`")));
			}
			input.parse::<Token![=]>()?;
			input.parse::<syn::LitBool>()?.value
		} else {
			default
		},
	)
}
