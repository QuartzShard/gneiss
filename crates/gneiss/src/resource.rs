//! Images, fonts, vector graphics and data bundled with the app.
//!
//! Declare each resource in `Cargo.toml`. The build compiles it for every platform the app
//! supports, packs it into the bundle, and generates a `resources` module with one typed key per
//! declaration, named after it:
//!
//! ```toml
//! [package.metadata.pebble.resources.LOGO]
//! type = "png"
//! file = "logo.png"
//! target_platforms = ["emery", "basalt"]
//! ```
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::resource::image::GBitMap;
//! # mod resources { use gneiss::resource::*; use core::num::NonZeroU32; pub const LOGO: ImageKey = ImageKey(NonZeroU32::new(1).unwrap()); }
//! let logo = GBitMap::load(resources::LOGO).expect("LOGO declared");
//! # }
//! ```
//!
//! A key's type follows the resource's `type`, so only the matching loader accepts it:
//!
//! | `type` | Key | Loaded by |
//! |---|---|---|
//! | `png`, `bitmap`, `pbi`, `icon` | [`ImageKey`] | [`GBitMap::load`](image::GBitMap::load) |
//! | `svg` (one file), `pdc` | [`PdcKey`] | [`DrawCommand::load`](image::DrawCommand::load) |
//! | `svg` (a directory of frames), animated `pdc` | [`PdcSeqKey`] | [`DrawCommandSequence::load`](image::DrawCommandSequence::load) |
//! | `font`, `pbf` | [`FontKey`] | [`Font::load`](font::Font::load) |
//! | `raw` | [`RawBytesKey`] | [`ResourceHandle::load`](raw::ResourceHandle::load), [`load_all`](raw::load_all) |
//! | `vibe` | [`VibeKey`] | not yet |
//!
//! `icon` also becomes the app's menu icon. `font` rasterises a TrueType or OpenType file at the
//! `pixelHeight` you give it, for the characters `characterRegex` matches.
//!
//! # Per-platform variants
//!
//! Siblings of `file` tagged after a `~` replace it on matching platforms: `logo~emery.png` on
//! emery, `logo~color.png` on any colour watch. A sibling tagged with the platform's own name wins;
//! otherwise the one carrying the most of the platform's tags (such as `color` or `round`);
//! otherwise the untagged file.
//!
//! # Every resource type
//!
//! Every entry needs `type`, `file` (relative to the app's `resources/` directory) and
//! `target_platforms`. The table's name becomes the key in the generated `resources` module.
//!
//! ## `raw`: bytes, copied as they are
//!
//! ```toml
//! [package.metadata.pebble.resources.DATA]
//! type = "raw"
//! file = "data.bin"
//! target_platforms = ["emery", "basalt"]
//! ```
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::{alloc::PebbleAlloc, resource::raw};
//! # mod resources { use gneiss::resource::*; use core::num::NonZeroU32; pub const DATA: RawBytesKey = RawBytesKey(NonZeroU32::new(1).unwrap()); }
//! let handle = raw::ResourceHandle::load(resources::DATA).expect("DATA declared");
//! for chunk in handle.iter::<64>() {
//!     // up to 64 bytes at a time
//! }
//! let whole = raw::load_all(resources::DATA, PebbleAlloc); // or all at once
//! # }
//! ```
//!
//! ## `png`, `bitmap`, `pbi`: images
//!
//! `png` and `bitmap` take any common image format. `png` keeps it a palettised PNG, decoded when
//! loaded; `bitmap` converts it to the firmware's raw bitmap format, larger but quicker to load.
//! `pbi` is an already-converted bitmap, copied as it is.
//!
//! ```toml
//! [package.metadata.pebble.resources.LOGO]
//! type = "png"   # or "bitmap", or "pbi" for a .pbi file
//! file = "logo.png"
//! target_platforms = ["emery", "basalt"]
//! ```
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::{resource::image::GBitMap, ui::{BitmapLayer, graphics::*}};
//! # mod resources { use gneiss::resource::*; use core::num::NonZeroU32; pub const LOGO: ImageKey = ImageKey(NonZeroU32::new(1).unwrap()); }
//! # let frame = Rectangle::new(Point::zero(), Size::new(100, 100));
//! let mut layer = BitmapLayer::new(frame).expect("layer");
//! layer.set_bitmap(GBitMap::load(resources::LOGO).expect("LOGO declared"));
//! # }
//! ```
//!
//! ## `icon`: the app's menu icon
//!
//! An image, as for `bitmap`, that the bundle also uses as the app's icon in the watch menu. It
//! needs no code.
//!
//! ```toml
//! [package.metadata.pebble.resources.APP_ICON]
//! type = "icon"
//! file = "icon.png"
//! target_platforms = ["emery", "basalt"]
//! ```
//!
//! ## `font`, `pbf`: fonts
//!
//! `font` rasterises a TrueType or OpenType file. `pbf` is an already-built Pebble font, copied as
//! it is.
//!
//! ```toml
//! [package.metadata.pebble.resources.FONT_ROBOTO_18]
//! type = "font"
//! file = "roboto.ttf"
//! target_platforms = ["emery", "basalt"]
//! pixelHeight = 18          # optional: defaults to the first number in the name, here 18
//! characterRegex = "[ -~]"  # optional: which characters to include, all by default
//! compress = true           # optional: smaller, slightly slower to draw
//! ```
//!
//! Also accepted: `characterList`, a JSON file with a `codepoints` array, applied together with
//! `characterRegex`; `trackingAdjust`, pixels added to every character's advance; and
//! `extended = true`, for an extra page of characters set on another font's baseline.
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::{resource::font::Font, ui::{TextLayer, graphics::*}};
//! # mod resources { use gneiss::resource::*; use core::num::NonZeroU32; pub const FONT_ROBOTO_18: FontKey = FontKey(NonZeroU32::new(1).unwrap()); }
//! # let frame = Rectangle::new(Point::zero(), Size::new(100, 100));
//! let mut text: TextLayer = TextLayer::new(frame).expect("layer");
//! text.set_font(Font::load(resources::FONT_ROBOTO_18)).set_text(c"Hello");
//! # }
//! ```
//!
//! ## `svg`, `pdc`: vector images and animations
//!
//! `svg` compiles an SVG to a draw command image. Given a directory instead of a file, it compiles
//! the SVGs inside, in name order, to the frames of an animation. `pdc` is an already-compiled
//! draw command image or animation, copied as it is.
//!
//! ```toml
//! [package.metadata.pebble.resources.FERRIS]
//! type = "svg"
//! file = "ferris.svg"
//! target_platforms = ["emery", "basalt"]
//!
//! [package.metadata.pebble.resources.SPINNER]
//! type = "svg"
//! file = "spinner"        # a directory of frames
//! target_platforms = ["emery", "basalt"]
//! frameDurationMs = 60    # optional
//! playCount = 65535       # optional: 65535 loops forever
//! tolerance = 0.5         # optional: how closely straight lines follow curves, in pixels
//! ```
//!
//! `precise` (optional) controls whether paths use sub-pixel precision: on by default for images
//! and off for animations.
//!
//! Draw them from a [`DataLayer`](crate::ui::DataLayer) that owns them:
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::{resource::image::*, ui::{DataLayer, graphics::*}};
//! # mod resources { use gneiss::resource::*; use core::num::NonZeroU32; pub const FERRIS: PdcKey = PdcKey(NonZeroU32::new(1).unwrap()); pub const SPINNER: PdcSeqKey = PdcSeqKey(NonZeroU32::new(1).unwrap()); }
//! # let frame = Rectangle::new(Point::zero(), Size::new(100, 100));
//! let image = DrawCommand::load(resources::FERRIS).expect("FERRIS declared");
//! let mut layer = DataLayer::new_with_data(frame, image).expect("layer");
//! layer.set_draw(|image, _bounds, ctx| ctx.draw_pdc(image, Point::zero()));
//!
//! let spinner = DrawCommandSequence::load(resources::SPINNER).expect("SPINNER declared");
//! // in a DrawFn: ctx.draw_pdc_sequence(&spinner, elapsed_ms, Point::zero())
//! # }
//! ```
//!
//! ## `vibe`: vibration patterns
//!
//! Compiles, and gets a key, but the firmware has no way to play one yet.
//!
//! ```toml
//! [package.metadata.pebble.resources.BUZZ]
//! type = "vibe"
//! file = "buzz.json"
//! target_platforms = ["emery", "basalt"]
//! ```
use core::num::NonZeroU32;

pub mod font;
pub mod image;
pub mod persist;
pub mod raw;

/// A bitmap image resource.
#[derive(Debug, Clone, Copy)]
pub struct ImageKey(pub NonZeroU32);
/// An animated PNG resource. No resource `type` produces one yet.
#[derive(Debug, Clone, Copy)]
pub struct AnimatedImageKey(pub NonZeroU32);

// both actually just PDC, but we know which ones are sequences at build time

/// A draw command (vector) image resource.
#[derive(Debug, Clone, Copy)]
pub struct PdcKey(pub NonZeroU32);
/// A draw command (vector) animation resource.
#[derive(Debug, Clone, Copy)]
pub struct PdcSeqKey(pub NonZeroU32);

/// A font resource.
#[derive(Debug, Clone, Copy)]
pub struct FontKey(pub NonZeroU32);

/// A raw bytes resource.
#[derive(Debug, Clone, Copy)]
pub struct RawBytesKey(pub NonZeroU32);

/// A vibration pattern resource. The firmware has no way to play one yet.
#[derive(Debug, Clone, Copy)]
pub struct VibeKey(pub NonZeroU32);
