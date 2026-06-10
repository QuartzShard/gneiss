use std::collections::{BTreeMap, BTreeSet};

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
		bit_depth: u8,
	},
}

#[repr(u8)]
// We want to match the upstream SDK here
#[allow(clippy::enum_variant_names)]
enum GBitMapFormat {
	Raw1Bit,
	Raw8Bit,
	Palette1Bit,
	Palette2Bit,
	Palette4Bit,
}

impl From<&PalettisedImage> for GBitMapFormat {
	fn from(value: &PalettisedImage) -> Self {
		match value {
			PalettisedImage::Monochrome { .. } => Self::Raw1Bit,
			PalettisedImage::ColourPalettised { bit_depth, .. } => match bit_depth {
				1 => Self::Palette1Bit,
				2 => Self::Palette2Bit,
				4 => Self::Palette4Bit,
				8 => Self::Raw8Bit,
				_ => panic!("Invalid palette bit depth"),
			},
		}
	}
}

// Map ARGB8 to ARGB2
fn quantise_image(file: &std::path::PathBuf, platform: Platform) -> Result<QuantisedImage> {
	let img = image::open(file)?;
	let mut palette = BTreeSet::new();

	Ok(if platform.config().capabilities.color {
		let bytes = img
			.to_rgba8()
			.as_bytes()
			.iter()
			.map(|px_component| (*px_component as u16 + 42) / 85)
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
				17..=255 => 8,
				5..=16 => 4,
				3..=4 => 2,
				1..=2 => 1,
				0 => bail!("No colours?"),
			};
			map_palette(bytes, size, palette, bit_depth)
		}
	})
}

fn map_palette(
	bytes: Vec<u8>,
	size: Resolution,
	palette: BTreeSet<u8>,
	bit_depth: u8,
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
		palette: palette.keys().cloned().collect(),
		bit_depth,
	}
}

pub(crate) fn process_png(file: &std::path::PathBuf, platform: Platform) -> Result<Vec<u8>> {
	let (indices, size, palette, bit_depth) = match palettise(quantise_image(file, platform)?)? {
		PalettisedImage::ColourPalettised {
			indices,
			size,
			palette,
			bit_depth,
		} => (indices, size, palette, bit_depth),
		PalettisedImage::Monochrome { bytes, size } => (bytes, size, vec![0xC0, 0xFF], 1),
	};

	let packed = pack_msb_first(indices, size, bit_depth);

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
		1 => png::BitDepth::One,
		2 => png::BitDepth::Two,
		4 => png::BitDepth::Four,
		8 => png::BitDepth::Eight,
		_ => bail!("Invalid palette bit depth"),
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

fn pack_msb_first(indices: Vec<u8>, size: Pair<u32>, bit_depth: u8) -> Vec<u8> {
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

pub(crate) fn process_bitmap(file: &std::path::PathBuf, platform: Platform) -> Result<Vec<u8>> {
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
							word |= (*px as u32) << i;
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
		} if bit_depth < 8 => {
			let indices_packed = pack_msb_first(indices, size, bit_depth);
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
		info_flags: 1 << 12 | ((format as u8) << 1) as u16,
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
		let mut palette_padded = vec![0; 1 << bit_depth];
		palette_padded[..palette.len()].copy_from_slice(&palette);
		buf.extend_from_slice(&palette_padded);
	};
	Ok(buf)
}

pub(crate) fn check_bitmap(file: &std::path::PathBuf) -> Result<Vec<u8>> {
	let img = std::fs::read(file)?;
	let header_len = size_of::<PebbleBitmapHeader>();
	if img.len() < header_len {
		bail!("PBI smaller than its header ({} < {header_len} bytes)", img.len());
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

	let format = (info >> 1) & 0x1F;
	let bit_depth: usize = match format {
		0 | 2 => 1,
		3 => 2,
		4 => 4,
		1 | 5 => 8,
		other => bail!("Unknown PBI format {other}"),
	};

	let w = header.bounds.size.x;
	let h = header.bounds.size.y;
	if w <= 0 || h <= 0 {
		bail!("Non-positive PBI bounds ({w}x{h})");
	}
	let (w, h) = (w as usize, h as usize);

	// 8-bit circular stores rows out-of-line; its geometry isn't a flat stride
	if format == 5 {
		return Ok(img);
	}

	// legacy 1-bit pads rows to a word, the palettised/8-bit formats to a byte
	let stride = if format == 0 {
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

	let palette_bytes = if matches!(format, 2 | 3 | 4) { 1 << bit_depth } else { 0 };
	let expected = header_len + stride * h + palette_bytes;
	if img.len() != expected {
		bail!(
			"PBI is {} bytes; header + {stride}x{h} + {palette_bytes} palette = {expected}",
			img.len()
		);
	}

	Ok(img)
}
