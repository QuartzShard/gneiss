//! Per-platform build/runtime configuration.
//!
//! Hand-transcribed from upstream
//! `~/.pebble-sdk/SDKs/current/sdk-core/pebble/common/tools/pebble_sdk_platform.py`.
//! Process-info platform bit values are from
//! `PebbleOS/src/fw/process_management/pebble_process_info.h:42-50`
//! (`PROCESS_INFO_PLATFORM_<NAME> = N << 6`; we store the raw N).
//!
//! The `defines` slice duplicates information also present in `capabilities`
//! and `display`. A unit test in this module asserts the two views agree, so
//! drift introduced when adding a new platform or capability is caught at
//! `cargo test -p pebble-types` time.

use crate::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayShape {
	Rect,
	Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Display {
	pub width:  u16,
	pub height: u16,
	pub shape:  DisplayShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capabilities {
	/// `false` means BW (`PBL_BW`); `true` means color (`PBL_COLOR`).
	pub color:            bool,
	pub microphone:       bool,
	pub smartstrap:       bool,
	/// Implies `smartstrap`. `PBL_SMARTSTRAP_POWER`.
	pub smartstrap_power: bool,
	pub health:           bool,
	pub compass:          bool,
	pub touch:            bool,
	pub rgb_backlight:    bool,
	pub speaker:          bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PlatformConfig {
	/// Upper-case canonical name, e.g. `"EMERY"`.
	pub name: &'static str,
	/// Raw `N` from `PROCESS_INFO_PLATFORM_<NAME> = N << 6`. The OS shifts it
	/// into position; we store the unshifted value to make the relationship
	/// explicit at use-time.
	pub process_info_platform_bit: u8,
	pub max_app_binary_size: u32,
	/// Becomes `LENGTH` of the `APP` memory region in the app linker script.
	pub max_app_memory_size: u32,
	pub max_worker_memory_size: u32,
	pub max_resources_size: u32,
	pub max_resources_size_appstore: u32,
	pub max_font_glyph_size: u16,
	pub display: Display,
	/// `Some(rev)` only for legacy platforms whose ABI is frozen.
	/// Mirrors the presence of `PBL_SDK_FROZEN` in `defines`.
	pub frozen_at_revision: Option<u32>,
	/// Bare define names, ready for `-D` prefixing. Includes the platform
	/// name (`PBL_PLATFORM_<NAME>`), color/shape (`PBL_COLOR`|`PBL_BW`,
	/// `PBL_RECT`|`PBL_ROUND`), capability flags, `PBL_DISPLAY_WIDTH=<n>` /
	/// `PBL_DISPLAY_HEIGHT=<n>`, and `PBL_SDK_FROZEN` where applicable.
	/// `PBL_SDK_3` is universal and is added by the consumer (xtask), not here.
	pub defines: &'static [&'static str],
	pub capabilities: Capabilities,
}

pub const PLATFORM_APLITE: PlatformConfig = PlatformConfig {
	name: "APLITE",
	process_info_platform_bit: 1,
	max_app_binary_size: 0x10000,
	max_app_memory_size: 0x6000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x80000,
	max_resources_size_appstore: 0x20000,
	max_font_glyph_size: 256,
	display: Display {
		width:  144,
		height: 168,
		shape:  DisplayShape::Rect,
	},
	frozen_at_revision: Some(81),
	defines: &[
		"PBL_PLATFORM_APLITE",
		"PBL_BW",
		"PBL_RECT",
		"PBL_COMPASS",
		"PBL_DISPLAY_WIDTH=144",
		"PBL_DISPLAY_HEIGHT=168",
		"PBL_SDK_FROZEN",
	],
	capabilities: Capabilities {
		color:            false,
		microphone:       false,
		smartstrap:       false,
		smartstrap_power: false,
		health:           false,
		compass:          true,
		touch:            false,
		rgb_backlight:    false,
		speaker:          false,
	},
};

pub const PLATFORM_BASALT: PlatformConfig = PlatformConfig {
	name: "BASALT",
	process_info_platform_bit: 2,
	max_app_binary_size: 0x10000,
	max_app_memory_size: 0x10000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 256,
	display: Display {
		width:  144,
		height: 168,
		shape:  DisplayShape::Rect,
	},
	frozen_at_revision: Some(89),
	defines: &[
		"PBL_PLATFORM_BASALT",
		"PBL_COLOR",
		"PBL_RECT",
		"PBL_MICROPHONE",
		"PBL_SMARTSTRAP",
		"PBL_HEALTH",
		"PBL_COMPASS",
		"PBL_SMARTSTRAP_POWER",
		"PBL_DISPLAY_WIDTH=144",
		"PBL_DISPLAY_HEIGHT=168",
		"PBL_SDK_FROZEN",
	],
	capabilities: Capabilities {
		color:            true,
		microphone:       true,
		smartstrap:       true,
		smartstrap_power: true,
		health:           true,
		compass:          true,
		touch:            false,
		rgb_backlight:    false,
		speaker:          false,
	},
};

pub const PLATFORM_CHALK: PlatformConfig = PlatformConfig {
	name: "CHALK",
	process_info_platform_bit: 3,
	max_app_binary_size: 0x10000,
	max_app_memory_size: 0x10000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 256,
	display: Display {
		width:  180,
		height: 180,
		shape:  DisplayShape::Round,
	},
	frozen_at_revision: Some(89),
	defines: &[
		"PBL_PLATFORM_CHALK",
		"PBL_COLOR",
		"PBL_ROUND",
		"PBL_MICROPHONE",
		"PBL_SMARTSTRAP",
		"PBL_HEALTH",
		"PBL_COMPASS",
		"PBL_SMARTSTRAP_POWER",
		"PBL_DISPLAY_WIDTH=180",
		"PBL_DISPLAY_HEIGHT=180",
		"PBL_SDK_FROZEN",
	],
	capabilities: Capabilities {
		color:            true,
		microphone:       true,
		smartstrap:       true,
		smartstrap_power: true,
		health:           true,
		compass:          true,
		touch:            false,
		rgb_backlight:    false,
		speaker:          false,
	},
};

pub const PLATFORM_DIORITE: PlatformConfig = PlatformConfig {
	name: "DIORITE",
	process_info_platform_bit: 4,
	max_app_binary_size: 0x10000,
	max_app_memory_size: 0x10000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 256,
	display: Display {
		width:  144,
		height: 168,
		shape:  DisplayShape::Rect,
	},
	frozen_at_revision: Some(89),
	defines: &[
		"PBL_PLATFORM_DIORITE",
		"PBL_BW",
		"PBL_RECT",
		"PBL_MICROPHONE",
		"PBL_HEALTH",
		"PBL_SMARTSTRAP",
		"PBL_DISPLAY_WIDTH=144",
		"PBL_DISPLAY_HEIGHT=168",
		"PBL_SDK_FROZEN",
	],
	capabilities: Capabilities {
		color:            false,
		microphone:       true,
		smartstrap:       true,
		smartstrap_power: false,
		health:           true,
		compass:          false,
		touch:            false,
		rgb_backlight:    false,
		speaker:          false,
	},
};

pub const PLATFORM_EMERY: PlatformConfig = PlatformConfig {
	name: "EMERY",
	process_info_platform_bit: 5,
	max_app_binary_size: 0x20000,
	max_app_memory_size: 0x20000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 512,
	display: Display {
		width:  200,
		height: 228,
		shape:  DisplayShape::Rect,
	},
	frozen_at_revision: None,
	defines: &[
		"PBL_PLATFORM_EMERY",
		"PBL_COLOR",
		"PBL_RECT",
		"PBL_MICROPHONE",
		"PBL_SMARTSTRAP",
		"PBL_HEALTH",
		"PBL_SMARTSTRAP_POWER",
		"PBL_COMPASS",
		"PBL_TOUCH",
		"PBL_RGB_BACKLIGHT",
		"PBL_SPEAKER",
		"PBL_DISPLAY_WIDTH=200",
		"PBL_DISPLAY_HEIGHT=228",
	],
	capabilities: Capabilities {
		color:            true,
		microphone:       true,
		smartstrap:       true,
		smartstrap_power: true,
		health:           true,
		compass:          true,
		touch:            true,
		rgb_backlight:    true,
		speaker:          true,
	},
};

pub const PLATFORM_FLINT: PlatformConfig = PlatformConfig {
	name: "FLINT",
	process_info_platform_bit: 6,
	max_app_binary_size: 0x10000,
	max_app_memory_size: 0x10000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 256,
	display: Display {
		width:  144,
		height: 168,
		shape:  DisplayShape::Rect,
	},
	frozen_at_revision: None,
	defines: &[
		"PBL_PLATFORM_FLINT",
		"PBL_BW",
		"PBL_RECT",
		"PBL_MICROPHONE",
		"PBL_HEALTH",
		"PBL_COMPASS",
		"PBL_SPEAKER",
		"PBL_DISPLAY_WIDTH=144",
		"PBL_DISPLAY_HEIGHT=168",
	],
	capabilities: Capabilities {
		color:            false,
		microphone:       true,
		smartstrap:       false,
		smartstrap_power: false,
		health:           true,
		compass:          true,
		touch:            false,
		rgb_backlight:    false,
		speaker:          true,
	},
};

pub const PLATFORM_GABBRO: PlatformConfig = PlatformConfig {
	name: "GABBRO",
	process_info_platform_bit: 7,
	max_app_binary_size: 0x20000,
	max_app_memory_size: 0x20000,
	max_worker_memory_size: 0x2800,
	max_resources_size: 0x0010_0000,
	max_resources_size_appstore: 0x40000,
	max_font_glyph_size: 512,
	display: Display {
		width:  260,
		height: 260,
		shape:  DisplayShape::Round,
	},
	frozen_at_revision: None,
	defines: &[
		"PBL_PLATFORM_GABBRO",
		"PBL_COLOR",
		"PBL_ROUND",
		"PBL_MICROPHONE",
		"PBL_HEALTH",
		"PBL_COMPASS",
		"PBL_TOUCH",
		"PBL_DISPLAY_WIDTH=260",
		"PBL_DISPLAY_HEIGHT=260",
	],
	capabilities: Capabilities {
		color:            true,
		microphone:       true,
		smartstrap:       false,
		smartstrap_power: false,
		health:           true,
		compass:          true,
		touch:            true,
		rgb_backlight:    false,
		speaker:          false,
	},
};

impl Platform {
	pub const fn config(self) -> &'static PlatformConfig {
		match self {
			Platform::Aplite => &PLATFORM_APLITE,
			Platform::Basalt => &PLATFORM_BASALT,
			Platform::Chalk => &PLATFORM_CHALK,
			Platform::Diorite => &PLATFORM_DIORITE,
			Platform::Emery => &PLATFORM_EMERY,
			Platform::Flint => &PLATFORM_FLINT,
			Platform::Gabbro => &PLATFORM_GABBRO,
		}
	}
}

#[cfg(test)]
mod tests {
	//! Drift tests: `defines` is hand-transcribed from upstream, while
	//! `capabilities`, `display`, and `frozen_at_revision` are structured
	//! views of the same data. These tests assert the two views agree, so
	//! a typo or missed entry in either fails loudly.

	extern crate std;
	use std::format;

	use strum::IntoEnumIterator;

	use super::*;

	fn has(defs: &[&str], needle: &str) -> bool {
		defs.contains(&needle)
	}

	#[test]
	fn platform_name_define_matches_struct_name() {
		for p in Platform::iter() {
			let cfg = p.config();
			let expected = format!("PBL_PLATFORM_{}", cfg.name);
			assert!(
				has(cfg.defines, &expected),
				"{}: missing {} in defines",
				cfg.name,
				expected
			);
		}
	}

	#[test]
	fn process_info_platform_bits_are_unique_and_1_through_7() {
		let mut seen = [false; 8];
		for p in Platform::iter() {
			let bit = p.config().process_info_platform_bit as usize;
			assert!(
				(1..=7).contains(&bit),
				"{}: bit {} outside 1..=7",
				p.config().name,
				bit
			);
			assert!(!seen[bit], "{}: bit {} duplicated", p.config().name, bit);
			seen[bit] = true;
		}
	}

	#[test]
	fn color_capability_matches_defines() {
		for p in Platform::iter() {
			let cfg = p.config();
			if cfg.capabilities.color {
				assert!(
					has(cfg.defines, "PBL_COLOR"),
					"{}: color=true but no PBL_COLOR",
					cfg.name
				);
				assert!(
					!has(cfg.defines, "PBL_BW"),
					"{}: color=true with PBL_BW",
					cfg.name
				);
			} else {
				assert!(
					has(cfg.defines, "PBL_BW"),
					"{}: color=false but no PBL_BW",
					cfg.name
				);
				assert!(
					!has(cfg.defines, "PBL_COLOR"),
					"{}: color=false with PBL_COLOR",
					cfg.name
				);
			}
		}
	}

	#[test]
	fn display_shape_matches_defines() {
		for p in Platform::iter() {
			let cfg = p.config();
			match cfg.display.shape {
				DisplayShape::Rect => {
					assert!(
						has(cfg.defines, "PBL_RECT"),
						"{}: shape=Rect but no PBL_RECT",
						cfg.name
					);
					assert!(
						!has(cfg.defines, "PBL_ROUND"),
						"{}: shape=Rect with PBL_ROUND",
						cfg.name
					);
				}
				DisplayShape::Round => {
					assert!(
						has(cfg.defines, "PBL_ROUND"),
						"{}: shape=Round but no PBL_ROUND",
						cfg.name
					);
					assert!(
						!has(cfg.defines, "PBL_RECT"),
						"{}: shape=Round with PBL_RECT",
						cfg.name
					);
				}
			}
		}
	}

	#[test]
	fn display_dimensions_match_defines() {
		for p in Platform::iter() {
			let cfg = p.config();
			let w = format!("PBL_DISPLAY_WIDTH={}", cfg.display.width);
			let h = format!("PBL_DISPLAY_HEIGHT={}", cfg.display.height);
			assert!(has(cfg.defines, &w), "{}: missing {}", cfg.name, w);
			assert!(has(cfg.defines, &h), "{}: missing {}", cfg.name, h);
		}
	}

	#[test]
	fn capability_bools_match_defines() {
		let pairs: &[(&str, fn(&Capabilities) -> bool)] = &[
			("PBL_MICROPHONE", |c| c.microphone),
			("PBL_SMARTSTRAP", |c| c.smartstrap),
			("PBL_SMARTSTRAP_POWER", |c| c.smartstrap_power),
			("PBL_HEALTH", |c| c.health),
			("PBL_COMPASS", |c| c.compass),
			("PBL_TOUCH", |c| c.touch),
			("PBL_RGB_BACKLIGHT", |c| c.rgb_backlight),
			("PBL_SPEAKER", |c| c.speaker),
		];
		for p in Platform::iter() {
			let cfg = p.config();
			for (define, getter) in pairs {
				let bool_set = getter(&cfg.capabilities);
				let define_present = has(cfg.defines, define);
				assert_eq!(
					bool_set, define_present,
					"{}: capability for {} disagrees with defines (struct={}, defines={})",
					cfg.name, define, bool_set, define_present
				);
			}
		}
	}

	#[test]
	fn smartstrap_power_implies_smartstrap() {
		for p in Platform::iter() {
			let c = p.config().capabilities;
			if c.smartstrap_power {
				assert!(
					c.smartstrap,
					"{}: smartstrap_power without smartstrap",
					p.config().name
				);
			}
		}
	}

	#[test]
	fn frozen_at_revision_matches_sdk_frozen_define() {
		for p in Platform::iter() {
			let cfg = p.config();
			let define_present = has(cfg.defines, "PBL_SDK_FROZEN");
			assert_eq!(
				cfg.frozen_at_revision.is_some(),
				define_present,
				"{}: frozen_at_revision ({:?}) disagrees with PBL_SDK_FROZEN in defines ({})",
				cfg.name,
				cfg.frozen_at_revision,
				define_present
			);
		}
	}
}
