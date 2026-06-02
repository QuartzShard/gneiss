// `clap`'s derive macro emits `::std::...` paths, so the `clap` feature forces std.
// `serde` is no_std-friendly when its default `std` feature is disabled; we leave
// `serde` defaults on for the host consumers and don't try to thread that needle.
// pebble-sys and other no_std consumers should depend on pebble-types without
// features — that path stays no_std.
#![cfg_attr(not(any(feature = "clap", feature = "serde")), no_std)]

use core::fmt;

mod config;
pub use config::{
    Capabilities, Display, DisplayShape, PLATFORM_APLITE, PLATFORM_BASALT, PLATFORM_CHALK,
    PLATFORM_DIORITE, PLATFORM_EMERY, PLATFORM_FLINT, PLATFORM_GABBRO, PlatformConfig,
};

/// Application target platforms, by the internal name. Aliased to the hardware
/// codenames that implement that platform, so you can identify your watch either way.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, strum::EnumIter)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Platform {
    /// Pebble Time 2. Marked as default because @quarzshard has one
    #[default]
    // Point of interest: Emery was also the target name for the ill-fated 2016 Time 2, codename
    // "robert". We don't claim support for it here because building against the modern SDK would
    // almost certainly crash your original Time 2 dev hardware, if you somehow happened to have
    // it. If you do, please get in touch, I'd love to chat!
    #[cfg_attr(feature = "clap", value(alias = "obelix"))]
    Emery,
    /// Pebble 2 Duo
    #[cfg_attr(feature = "clap", value(alias = "asterix"))]
    Flint,
    /// Pebble Round 2
    #[cfg_attr(feature = "clap", value(alias = "getafix"))]
    Gabbro,
    /// Pebble Classic / Steel
    #[cfg_attr(feature = "clap", value(aliases = ["tintin", "bianca"]))]
    Aplite,
    /// Pebble Time / Time Steel
    #[cfg_attr(feature = "clap", value(alias = "snowy"))]
    Basalt,
    /// Pebble Time Round
    #[cfg_attr(feature = "clap", value(alias = "spalding"))]
    Chalk,
    /// Pebble 2
    #[cfg_attr(feature = "clap", value(alias = "silk"))]
    Diorite,
}

impl Platform {
    /// `rustc`/LLVM target triple for compiling Pebble *app* code. Universal
    /// across all platforms: apps run inside the OS sandbox and only ever see
    /// the cortex-m3/thumbv7m ABI, regardless of the underlying SoC (the new
    /// SF32LB52x-based boards still expose a thumbv7m app ABI through the
    /// syscall trampoline).
    pub const fn app_target_triple(self) -> &'static str {
        "thumbv7m-none-eabi"
    }

    /// Lowercase canonical name (matches `Display` and `serde` representation).
    pub const fn as_str(self) -> &'static str {
        match self {
            Platform::Emery => "emery",
            Platform::Flint => "flint",
            Platform::Gabbro => "gabbro",
            Platform::Aplite => "aplite",
            Platform::Basalt => "basalt",
            Platform::Chalk => "chalk",
            Platform::Diorite => "diorite",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returned when a string does not name a known platform or alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownPlatform;

impl fmt::Display for UnknownPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a known platform or hardware alias")
    }
}

impl core::error::Error for UnknownPlatform {}

impl core::str::FromStr for Platform {
    type Err = UnknownPlatform;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Match the canonical lowercase names and the clap-side hardware aliases
        // so a single source of truth covers both `clap` arg parsing and any
        // ad-hoc `s.parse()` callers (e.g. pebble-sys's build script).
        match s {
            "emery" | "obelix" => Ok(Self::Emery),
            "flint" | "asterix" => Ok(Self::Flint),
            "gabbro" | "getafix" => Ok(Self::Gabbro),
            "aplite" | "tintin" | "bianca" => Ok(Self::Aplite),
            "basalt" | "snowy" => Ok(Self::Basalt),
            "chalk" | "spalding" => Ok(Self::Chalk),
            "diorite" | "silk" => Ok(Self::Diorite),
            _ => Err(UnknownPlatform),
        }
    }
}
