//! Platform selection for build scripts: which platform feature is enabled, and the cfg flags
//! it brings.
use std::env::var;

use strum::IntoEnumIterator;

use crate::{ALL_CFGS, Platform};

/// Cargo sets `CARGO_FEATURE_<NAME>` for each enabled feature, so a platform's
/// selection env var follows from its name.
pub fn platform_features() -> impl Iterator<Item = (String, Platform)> {
	Platform::iter().map(|platform| {
		(
			format!("CARGO_FEATURE_{}", platform.as_str().to_uppercase()),
			platform,
		)
	})
}

/// Pick the target platform.
/// `mutually_exclusive_features` only enforces at lib compile time, and build.rs
/// runs first, so do the check here too.
/// # Panics
/// If multiple platforms are enabled by build time, or if the enabled platform is invalid
#[allow(clippy::panic)]
pub fn select_platform() -> Platform {
	let enabled: Vec<Platform> = platform_features()
		.filter_map(|(env, platform)| var(env).ok().map(|_| platform))
		.collect();
	match enabled.as_slice() {
		[p] => *p,
		[] => var("PBL_PLATFORM")
			.unwrap_or_else(|_| {
				panic!(
					"no platform selected: enable exactly one of the {} features, or set \
					 PBL_PLATFORM",
					Platform::iter()
						.map(|p| format!("`{}`", p.as_str()))
						.collect::<Vec<_>>()
						.join("/")
				)
			})
			.parse()
			.expect("invalid PBL_PLATFORM"),
		many => panic!(
			"multiple platforms enabled ({:?}); choose exactly one",
			many.iter().map(|p| p.as_str()).collect::<Vec<_>>()
		),
	}
}

/// Set the platform's `pbl_*` cfg flags, and declare the whole vocabulary so
/// `#[cfg]`-ing on a flag this platform lacks doesn't trip `unexpected_cfgs`.
/// A build script's cfgs reach only its own crate, so every crate that gates
/// on them calls this.
pub fn emit_platform_cfgs(platform: Platform) {
	for flag in ALL_CFGS {
		println!("cargo::rustc-check-cfg=cfg({flag})");
	}
	for flag in platform.config().cfgs {
		println!("cargo::rustc-cfg={flag}");
	}
}
