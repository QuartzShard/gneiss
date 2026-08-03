use std::{
	collections::{BTreeMap, BTreeSet},
	path::Path,
};

use bytemuck::{Pod, Zeroable, pod_read_unaligned};
use color_eyre::eyre::{OptionExt, Result, bail};
use image::EncodableLayout;
use pebble_types::Platform;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
struct Pair<T> {
	x: T,
	y: T,
}

impl<T> Pair<T> {
	fn try_map<U: TryInto<T>>(value: Pair<U>) -> Option<Self> {
		Some(Pair {
			x: value.x.try_into().ok()?,
			y: value.y.try_into().ok()?,
		})
	}
}
type Resolution = Pair<u32>;

enum QuantisedImage {
	Colour {
		bytes:   Vec<u8>,
		size:    Resolution,
		palette: BTreeSet<u8>,
	},
	Monochrome {
		bytes: Vec<u8>,
		size:  Resolution,
	},
}

/// Not Bit-packed, as that's output-format independent. Passes through Monochrome directly.
/// Constructs palette table and maps colour -> Index for palettised colour images.
enum PalettisedImage {
	Monochrome {
		bytes: Vec<u8>,
		size:  Resolution,
	},
	ColourPalettised {
		indices:   Vec<u8>,
		size:      Resolution,
		palette:   Vec<u8>,
		bit_depth: BitDepth,
	},
}

#[derive(Copy, Clone)]
enum BitDepth {
	One   = 1,
	Two   = 2,
	Four  = 4,
	Eight = 8,
}

#[derive(strum::FromRepr)]
#[repr(u8)]
// We want to match the upstream SDK here
#[allow(clippy::enum_variant_names)]
enum GBitMapFormat {
	Raw1Bit,
	Raw8Bit,
	Palette1Bit,
	Palette2Bit,
	Palette4Bit,
	Circular8Bit,
}

impl GBitMapFormat {
	pub fn bit_depth(&self) -> usize {
		match self {
			GBitMapFormat::Raw1Bit | GBitMapFormat::Palette1Bit => 1,
			GBitMapFormat::Palette2Bit => 2,
			GBitMapFormat::Palette4Bit => 4,
			GBitMapFormat::Raw8Bit | GBitMapFormat::Circular8Bit => 8,
		}
	}
}

impl From<&PalettisedImage> for GBitMapFormat {
	fn from(value: &PalettisedImage) -> Self {
		match value {
			PalettisedImage::Monochrome { .. } => Self::Raw1Bit,
			PalettisedImage::ColourPalettised { bit_depth, .. } => match bit_depth {
				BitDepth::One => Self::Palette1Bit,
				BitDepth::Two => Self::Palette2Bit,
				BitDepth::Four => Self::Palette4Bit,
				BitDepth::Eight => Self::Raw8Bit,
			},
		}
	}
}

// Map ARGB8 to ARGB2
fn quantise_image(file: &Path, platform: Platform) -> Result<QuantisedImage> {
	let img = image::open(file)?;

	let mut palette = BTreeSet::new();
	Ok(if platform.config().capabilities.color {
		let bytes = img
			.to_rgba8()
			.as_bytes()
			.iter()
			.map(|px_component| (u16::from(*px_component) + 42) / 85)
			// RGBA8: px = [u8;4]; divides clean
			.array_chunks::<4>()
			.map(|px| {
				// Zero-out transparent pixels
				if px[3] != 0 {
					(px[3] << 6 | px[0] << 4 | px[1] << 2 | px[2]) as u8
				} else {
					0
				}
			})
			.inspect(|px| {
				// Keep track of how many colours we've used (surprise tool that will help us later)
				palette.insert(*px);
			})
			.collect();
		QuantisedImage::Colour {
			bytes,
			size: Resolution {
				x: img.width(),
				y: img.height(),
			},
			palette,
		}
	} else {
		let bytes = img
			.to_luma8()
			.as_bytes()
			.iter()
			.map(|px| *px >> 7)
			.collect();
		QuantisedImage::Monochrome {
			bytes,
			size: Resolution {
				x: img.width(),
				y: img.height(),
			},
		}
	})
}

fn palettise(img: QuantisedImage) -> Result<PalettisedImage> {
	Ok(match img {
		QuantisedImage::Monochrome { bytes, size } => PalettisedImage::Monochrome { bytes, size },
		QuantisedImage::Colour {
			bytes,
			size,
			palette,
		} => {
			// Palettise to smallest possible
			let bit_depth = match palette.len() {
				// Unreachable for u8, but palette.len() is usize, so
				256.. => bail!("Too many colours"),
				17..=255 => BitDepth::Eight,
				5..=16 => BitDepth::Four,
				3..=4 => BitDepth::Two,
				1..=2 => BitDepth::One,
				0 => bail!("No colours?"),
			};
			map_palette(&bytes, size, &palette, bit_depth)
		}
	})
}

fn map_palette(
	bytes: &[u8],
	size: Resolution,
	palette: &BTreeSet<u8>,
	bit_depth: BitDepth,
) -> PalettisedImage {
	// Pixel -> index map
	let palette = BTreeMap::from_iter(palette.iter().enumerate().map(|(i, px)| (*px, i as u8)));
	let indices = bytes
		.iter()
		.map(|px| *palette.get(px).expect("Colour wasn't in palette!"))
		.collect();
	PalettisedImage::ColourPalettised {
		indices,
		size,
		palette: palette.keys().copied().collect(),
		bit_depth,
	}
}

pub(crate) fn process_png(file: &Path, platform: Platform) -> Result<Vec<u8>> {
	let (indices, size, palette, bit_depth) = match palettise(quantise_image(file, platform)?)? {
		PalettisedImage::ColourPalettised {
			indices,
			size,
			palette,
			bit_depth,
		} => (indices, size, palette, bit_depth),
		PalettisedImage::Monochrome { bytes, size } => {
			(bytes, size, vec![0xC0, 0xFF], BitDepth::One)
		}
	};

	let packed = pack_msb_first(&indices, size, bit_depth as u8);

	let plte: Vec<u8> = palette
		.iter()
		.flat_map(|px| [((px >> 4) & 3) * 85, ((px >> 2) & 3) * 85, (px & 3) * 85])
		.collect();

	// tRNS may be shorter than PLTE; the sort puts opaque entries last, so trailing
	// fully-opaque alphas are dropped (entries past tRNS are treated as opaque)
	let trns: Vec<u8> = palette
		.iter()
		.map(|px| ((px >> 6) & 3) * 85)
		.filter(|alpha| *alpha < 255)
		.collect();

	let depth = match bit_depth {
		BitDepth::One => png::BitDepth::One,
		BitDepth::Two => png::BitDepth::Two,
		BitDepth::Four => png::BitDepth::Four,
		BitDepth::Eight => png::BitDepth::Eight,
	};

	let mut out = Vec::new();
	let mut encoder = png::Encoder::new(&mut out, size.x, size.y);
	encoder.set_color(png::ColorType::Indexed);
	encoder.set_depth(depth);
	encoder.set_palette(plte);
	if !trns.is_empty() {
		encoder.set_trns(trns);
	}
	encoder.set_compression(png::Compression::High);
	let mut writer = encoder.write_header()?;
	writer.write_image_data(&packed)?;
	writer.finish()?;
	Ok(out)
}

fn pack_msb_first(indices: &[u8], size: Pair<u32>, bit_depth: u8) -> Vec<u8> {
	let packed: Vec<u8> = indices
		.chunks(size.x as usize)
		.flat_map(|row| {
			row.chunks(8 / bit_depth as usize).map(|pxs| {
				let mut byte = 0u8;
				for (i, px) in pxs.iter().enumerate() {
					byte |= px << (8 - (i + 1) * bit_depth as usize);
				}
				byte
			})
		})
		.collect();
	packed
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct PebbleBitmapHeader {
	row_size_bytes: u16,
	info_flags:     u16,
	bounds:         BitmapBounds,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct BitmapBounds {
	origin: Pair<i16>,
	size:   Pair<i16>,
}

/// PebbleOS's launcher glance loader rejects app icons above this size
/// (`ATTRIBUTE_ICON_TINY_SIZE_PX` in `app_glance_structured.h`) — but only
/// after fully decoding them into a heap-allocated bitmap first. An
/// oversized icon still causes that transient allocation on every launcher
/// row draw, which is enough to crash the launcher when the app's row
/// scrolls into view. Enforce the cap at build time instead.
const MAX_ICON_SIZE: u32 = 25;

pub(crate) fn process_icon(file: &Path, platform: Platform) -> Result<Vec<u8>> {
	let (w, h) = image::image_dimensions(file)?;
	if w > MAX_ICON_SIZE || h > MAX_ICON_SIZE {
		bail!(
			"Icon is {w}x{h}, but app icons must be at most {MAX_ICON_SIZE}x{MAX_ICON_SIZE} \
			 (PebbleOS's launcher glance loader rejects anything larger, but not before \
			 fully decoding it — an oversized icon can crash the launcher)"
		);
	}
	process_bitmap(file, platform)
}

pub(crate) fn process_bitmap(file: &Path, platform: Platform) -> Result<Vec<u8>> {
	let img = palettise(quantise_image(file, platform)?)?;
	let format = GBitMapFormat::from(&img);
	let (bytes, size, palette, row_size_bytes) = match img {
		PalettisedImage::Monochrome { bytes, size } => {
			let packed: Vec<u8> = bytes
				.chunks(size.x as usize)
				.flat_map(|row| {
					row.chunks(32).flat_map(|pxs| {
						let mut word: u32 = 0;
						for (i, px) in pxs.iter().enumerate() {
							word |= u32::from(*px) << i;
						}
						word.to_le_bytes()
					})
				})
				.collect();
			(packed, size, None, (size.x.div_ceil(32) * 4) as u16)
		}
		PalettisedImage::ColourPalettised {
			indices,
			size,
			palette,
			bit_depth,
		} if (bit_depth as u8) < 8 => {
			let indices_packed = pack_msb_first(&indices, size, bit_depth as u8);
			(
				indices_packed,
				size,
				Some((palette, bit_depth)),
				(size.x * bit_depth as u32).div_ceil(8) as u16,
			)
		}
		// This needs to be un-palettised back to raw 8bit
		PalettisedImage::ColourPalettised {
			indices,
			size,
			palette,
			..
		} => {
			let bytes = indices.iter().map(|i| palette[*i as usize]).collect();
			(bytes, size, None, size.x as u16)
		}
	};
	let header = PebbleBitmapHeader {
		row_size_bytes,
		info_flags: 1 << 12 | u16::from((format as u8) << 1),
		bounds: BitmapBounds {
			origin: Pair { x: 0, y: 0 },
			size:   Pair::try_map(size).ok_or_eyre("Size does not fit in i16")?,
		},
	};

	let mut buf: Vec<u8> = Vec::with_capacity(
		row_size_bytes as usize * size.y as usize + size_of::<PebbleBitmapHeader>() + 16,
	);
	buf.extend_from_slice(bytemuck::bytes_of(&header));
	buf.extend_from_slice(&bytes);
	if let Some((palette, bit_depth)) = palette {
		let mut palette_padded = vec![0; 1 << (bit_depth as u8)];
		palette_padded[..palette.len()].copy_from_slice(&palette);
		buf.extend_from_slice(&palette_padded);
	}
	Ok(buf)
}

pub(crate) fn check_bitmap(file: &Path) -> Result<Vec<u8>> {
	check_bitmap_bytes(std::fs::read(file)?)
}

fn check_bitmap_bytes(img: Vec<u8>) -> Result<Vec<u8>> {
	let header_len = size_of::<PebbleBitmapHeader>();
	if img.len() < header_len {
		bail!(
			"PBI smaller than its header ({} < {header_len} bytes)",
			img.len()
		);
	}
	// Vec data isn't guaranteed aligned for the header's u16 fields
	let header: PebbleBitmapHeader = pod_read_unaligned(&img[..header_len]);

	let info = header.info_flags;
	if info & 1 != 0 {
		bail!("PBI is_heap_allocated bit set; must be 0 for a stored bitmap");
	}
	if (info >> 6) & 0x3F != 0 {
		bail!("PBI reserved info_flags bits are set");
	}
	let version = info >> 12;
	if version != 1 {
		bail!("Unexpected PBI version {version} (expected 1)");
	}

	let format_bits = ((info >> 1) & 0x1F) as u8;
	let format = GBitMapFormat::from_repr(format_bits)
		.ok_or_eyre(format!("Unknown GBitmapFormat {format_bits}"))?;

	let bit_depth = format.bit_depth();

	let w = header.bounds.size.x;
	let h = header.bounds.size.y;
	if w <= 0 || h <= 0 {
		bail!("Non-positive PBI bounds ({w}x{h})");
	}
	let (w, h) = (w as usize, h as usize);

	// circular stores rows out-of-line; its geometry isn't a flat stride
	if matches!(format, GBitMapFormat::Circular8Bit) {
		return Ok(img);
	}

	// legacy 1-bit pads rows to a word, the palettised/8-bit formats to a byte
	let stride = if matches!(format, GBitMapFormat::Raw1Bit) {
		w.div_ceil(32) * 4
	} else {
		(w * bit_depth).div_ceil(8)
	};
	if header.row_size_bytes as usize != stride {
		bail!(
			"PBI row_size_bytes {} != {stride} expected for {w}px @ {bit_depth}bpp",
			header.row_size_bytes
		);
	}

	let palette_bytes = if matches!(
		format,
		GBitMapFormat::Palette1Bit | GBitMapFormat::Palette2Bit | GBitMapFormat::Palette4Bit
	) {
		1 << bit_depth
	} else {
		0
	};
	let expected = header_len + stride * h + palette_bytes;
	if img.len() != expected {
		bail!(
			"PBI is {} bytes; header + {stride}x{h} + {palette_bytes} palette = {expected}",
			img.len()
		);
	}

	Ok(img)
}

#[cfg(test)]
mod tests {
	use strum::IntoEnumIterator;

	use super::*;

	const ICON: &str = concat!(
		env!("CARGO_MANIFEST_DIR"),
		"/../cargo-pebble/default_icon.png"
	);

	/// ARGB2 -> the RGBA8 that `process_png` writes into PLTE/tRNS.
	fn argb2_to_rgba(px: u8) -> [u8; 4] {
		[
			((px >> 4) & 3) * 85,
			((px >> 2) & 3) * 85,
			(px & 3) * 85,
			((px >> 6) & 3) * 85,
		]
	}

	/// Every platform's bit depth must survive `process_icon` -> `check_bitmap`,
	/// which re-derives stride and palette length from the encoded format.
	#[test]
	fn icon_round_trips_through_pbi() -> Result<()> {
		for platform in Platform::iter() {
			let pbi = process_icon(Path::new(ICON), platform)?;
			let checked =
				check_bitmap_bytes(pbi.clone()).unwrap_or_else(|e| panic!("{platform}: {e}"));
			assert_eq!(pbi, checked, "{platform}");
		}
		Ok(())
	}

	/// Decode the emitted PNG back and check every pixel against the palette
	/// entry `palettise` chose for it.
	#[test]
	fn icon_round_trips_through_png() -> Result<()> {
		for platform in Platform::iter() {
			let encoded = process_png(Path::new(ICON), platform)?;

			let (indices, palette) = match palettise(quantise_image(Path::new(ICON), platform)?)? {
				PalettisedImage::ColourPalettised {
					indices, palette, ..
				} => (indices, palette),
				PalettisedImage::Monochrome { bytes, .. } => (bytes, vec![0xC0, 0xFF]),
			};

			let mut decoder = png::Decoder::new(std::io::Cursor::new(&encoded));
			decoder.set_transformations(png::Transformations::EXPAND);
			let mut reader = decoder.read_info()?;
			let mut buf = vec![0; reader.output_buffer_size().expect("output buffer size")];
			let info = reader.next_frame(&mut buf)?;

			let channels = info.color_type.samples();
			assert_eq!(
				buf.len() / channels,
				indices.len(),
				"{platform}: pixel count"
			);
			for (i, (px, index)) in buf.chunks_exact(channels).zip(&indices).enumerate() {
				let expect = argb2_to_rgba(palette[*index as usize]);
				assert_eq!(px, &expect[..channels], "{platform}: pixel {i}");
			}
		}
		Ok(())
	}
}
