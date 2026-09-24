//! Drift tests for the generated `config.rs`.
//!
//! The table is generated from upstream's own platform data by `cargo xtask
//! regen-sys`, so these no longer guard a hand transcription — they guard the
//! generator, asserting that the structured views (`capabilities`, `display`,
//! `frozen_at_revision`, `cfgs`) still agree with the `defines` they were
//! derived from.

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

#[test]
fn tags_lead_with_the_platform_name() {
	for p in Platform::iter() {
		let cfg = p.config();
		assert_eq!(
			cfg.tags.first().copied(),
			Some(p.as_str()),
			"{}: tags should start with the platform name, got {:?}",
			cfg.name,
			cfg.tags
		);
	}
}

#[test]
fn cfgs_are_the_valueless_defines_lowercased() {
	for p in Platform::iter() {
		let cfg = p.config();
		let mut expected: std::vec::Vec<std::string::String> = cfg
			.defines
			.iter()
			.filter(|d| !d.contains('='))
			.map(|d| d.to_lowercase())
			.collect();
		expected.sort();
		assert_eq!(
			cfg.cfgs, expected,
			"{}: cfgs disagree with defines",
			cfg.name
		);
		for flag in cfg.cfgs {
			assert!(
				ALL_CFGS.contains(flag),
				"{}: {flag} missing from ALL_CFGS, so check-cfg would reject it",
				cfg.name
			);
		}
	}
}
