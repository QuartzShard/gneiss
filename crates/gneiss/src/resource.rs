//! Type-aware resource handles, generated per-app by pebble-build::gen_resource_delarations.
use core::num::NonZeroU32;

pub mod font;
pub mod image;
pub mod raw;

/// Handle to a static image. PNG/Bitmap storage is runtime irrelevant
#[derive(Debug, Clone, Copy)]
pub struct ImageKey(pub NonZeroU32);
/// Handle to an APNG
#[derive(Debug, Clone, Copy)]
pub struct AnimatedImageKey(pub NonZeroU32);

// both actually just PDC, but we know which ones are sequences at build time

/// Handle to a PDC Vector Image
#[derive(Debug, Clone, Copy)]
pub struct PdcKey(pub NonZeroU32);
/// Handle to a PDC amination sequence
#[derive(Debug, Clone, Copy)]
pub struct PdcSeqKey(pub NonZeroU32);

/// Handle to a font
#[derive(Debug, Clone, Copy)]
pub struct FontKey(pub NonZeroU32);

/// Handle to raw bytes
#[derive(Debug, Clone, Copy)]
pub struct RawBytesKey(pub NonZeroU32);

/// Not useful yet: no way to load this is exposed by the firmware
#[derive(Debug, Clone, Copy)]
pub struct VibeKey(pub NonZeroU32);
