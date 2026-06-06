#![no_std]

use pebble_types::{
	self,
	header::{DEFAULT_HEADER, PebbleProcessInfo},
};

mutually_exclusive_features::exactly_one_of!(
	"emery", "flint", "gabbro", "aplite", "basalt", "chalk", "diorite"
);

mod shim;

// Shut up, clippy! We don't care that bindgen outputs messy code
#[allow(
	non_camel_case_types,
	dead_code,
	non_upper_case_globals,
	unnecessary_transmutes,
	non_snake_case,
	unsafe_op_in_unsafe_fn,
	clippy::suspicious_doc_comments,
	clippy::useless_transmute,
	clippy::transmute_int_to_bool,
	clippy::upper_case_acronyms
)]
#[cfg_attr(feature = "emery", path = "platform/emery.rs")]
#[cfg_attr(feature = "flint", path = "platform/flint.rs")]
#[cfg_attr(feature = "gabbro", path = "platform/gabbro.rs")]
#[cfg_attr(feature = "aplite", path = "platform/aplite.rs")]
#[cfg_attr(feature = "basalt", path = "platform/basalt.rs")]
#[cfg_attr(feature = "chalk", path = "platform/chalk.rs")]
#[cfg_attr(feature = "diorite", path = "platform/diorite.rs")]
mod sys_raw;
pub use sys_raw::*;

#[used]
#[unsafe(link_section = ".pbl_header")]
pub static PBL_HEADER: PebbleProcessInfo = DEFAULT_HEADER;
