use strum::Display;

pub use crate::pebble_process_info::{
	COMPANY_NAME_BYTES, PROCESS_INFO_CRC_START_OFFSET, PROCESS_NAME_BYTES, PebbleProcessInfo,
	Version,
};
use crate::pebble_process_info::{
	PROCESS_INFO_CURRENT_SDK_VERSION_MAJOR, PROCESS_INFO_CURRENT_SDK_VERSION_MINOR,
	PROCESS_INFO_CURRENT_STRUCT_VERSION_MAJOR, PROCESS_INFO_CURRENT_STRUCT_VERSION_MINOR,
	PebbleProcessInfo__bindgen_ty_1,
};
pub const CURRENT_SDK_VERSION: Version = Version {
	major: PROCESS_INFO_CURRENT_SDK_VERSION_MAJOR as u8,
	minor: PROCESS_INFO_CURRENT_SDK_VERSION_MINOR as u8,
};
pub const CURRENT_HEADER_VERSION: Version = Version {
	major: PROCESS_INFO_CURRENT_STRUCT_VERSION_MAJOR as u8,
	minor: PROCESS_INFO_CURRENT_STRUCT_VERSION_MINOR as u8,
};

// Mostly Zeroed
impl Default for PebbleProcessInfo {
	fn default() -> Self {
		DEFAULT_HEADER
	}
}

// WARN: If the anonymous type bindgen creates for the inline named UUID thing in
// PebbleProcessInfo gets a different index, this must be changed to point at the correct type
pub type PblUUID = PebbleProcessInfo__bindgen_ty_1;
const _: () = assert!(size_of::<PblUUID>() == size_of::<[u8; 16]>());
impl From<[u8; 16]> for PblUUID {
	fn from(value: [u8; 16]) -> Self {
		uuid_from_array(value)
	}
}
#[derive(Debug, Display)]
pub enum PblUUIDError {
	WrongSize,
}

impl core::error::Error for PblUUIDError {}
impl TryFrom<&[u8]> for PblUUID {
	type Error = PblUUIDError;

	/// Error if `value.len()` != 16
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		Ok((*value.as_array::<16>().ok_or(PblUUIDError::WrongSize)?).into())
	}
}

const fn uuid_from_array(value: [u8; 16]) -> PblUUID {
	PblUUID {
		byte0:  value[0],
		byte1:  value[1],
		byte2:  value[2],
		byte3:  value[3],
		byte4:  value[4],
		byte5:  value[5],
		byte6:  value[6],
		byte7:  value[7],
		byte8:  value[8],
		byte9:  value[9],
		byte10: value[10],
		byte11: value[11],
		byte12: value[12],
		byte13: value[13],
		byte14: value[14],
		byte15: value[15],
	}
}
#[allow(
	clippy::useless_transmute,
	reason = "`c_char` is i8 on the host (x86-64) but unsigned on ARM, so this is \
	          a real reinterpret when linting for the host and a no-op when \
	          linting for thumbv7m-pebble. Only the latter trips the lint; the \
	          transmute has to stay for the host build to compile at all."
)]
pub const DEFAULT_HEADER: PebbleProcessInfo = PebbleProcessInfo {
	header:             unsafe {
		core::mem::transmute::<[u8; 8], [core::ffi::c_char; 8]>(*b"PBLAPP\0\0")
	},
	struct_version:     CURRENT_HEADER_VERSION,
	sdk_version:        CURRENT_SDK_VERSION,
	process_version:    Version { major: 0, minor: 0 },
	load_size:          0,
	offset:             0,
	crc:                0,
	name:               [0; PROCESS_NAME_BYTES as usize],
	company:            [0; COMPANY_NAME_BYTES as usize],
	icon_resource_id:   0,
	sym_table_addr:     0,
	flags:              0,
	num_reloc_entries:  0,
	uuid:               uuid_from_array([0; 16]),
	resource_crc:       0,
	resource_timestamp: 0,
	virtual_size:       0,
	load_size_hi:       0,
	virtual_size_hi:    0,
};
