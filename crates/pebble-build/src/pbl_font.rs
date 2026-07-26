//! Rasterise fonts
use std::{
	collections::{BTreeMap, HashSet},
	path::{Path, PathBuf},
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail},
};
use freetype::face::LoadFlag;
use pebble_types::Platform;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FONT_VERSION: u8 = 3;
const WILDCARD: char = '\u{25AF}';
const ELLIPSIS: char = '\u{2026}';
const MIN_CODEPOINT: u32 = 0x20;
const HASH_TABLE_SIZE: usize = 255;
const OFFSET_TABLE_MAX_SIZE: usize = 128;
const MAX_GLYPHS: usize = 256;
const MAX_GLYPHS_EXTENDED: usize = HASH_TABLE_SIZE * OFFSET_TABLE_MAX_SIZE;
const FEATURE_OFFSET_16: u8 = 0x01;
const FEATURE_RLE4: u8 = 0x02;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSettings {
	#[serde(default)]
	character_regex: Option<String>,
	#[serde(default)]
	character_list:  Option<PathBuf>,
	#[serde(default)]
	tracking_adjust: Option<i8>,
	#[serde(default)]
	pixel_height:    Option<u16>,
	#[serde(default)]
	compress:        Option<bool>,
	#[serde(default)]
	extended:        bool,
}

#[repr(C)]
struct GlyphHeader {
	width:       u8,
	height:      u8,
	offset_left: i8,
	offset_top:  i8,
	advance:     i8,
}

impl GlyphHeader {
	fn to_bytes(&self) -> [u8; 5] {
		[
			self.width,
			self.height,
			self.offset_left as u8,
			self.offset_top as u8,
			self.advance as u8,
		]
	}
}

struct EncodedGlyph {
	header: GlyphHeader,
	body:   Vec<u8>,
}

pub fn process_font(
	file: &Path,
	name: &str,
	platform: Platform,
	settings: FontSettings,
) -> Result<Vec<u8>> {
	let ft = freetype::Library::init()?;
	let typeface = ft.new_face(file, 0)?;
	let size =
		get_font_size(&settings, name).ok_or_eyre("Font definition did not specify a size")?;
	typeface.set_pixel_sizes(0, size as u32)?;

	// Extended pages sit on the base font's baseline, taken from the resource name
	let baseline = if settings.extended {
		name_height(name).ok_or_eyre("Extended font name must carry the base font's size")?
	} else {
		size
	};
	let max_glyphs = if settings.extended {
		MAX_GLYPHS_EXTENDED
	} else {
		MAX_GLYPHS
	};
	let (subset, index_map) = get_subset(&typeface, &settings, max_glyphs)?;
	let glyphs = index_map
		.into_iter()
		.map(|gindex| {
			encode_glyph(
				&typeface,
				gindex,
				&settings,
				baseline,
				platform.config().max_font_glyph_size as usize,
			)
		})
		.collect::<Result<Vec<_>>>()?;
	let (glyph_table, offsets) = layout(&glyphs);

	let mut entries: Vec<(u32, u32)> = subset
		.into_iter()
		.map(|(char, slot)| (char as u32, offsets[slot]))
		.collect();
	entries.sort_unstable_by_key(|(codepoint, _)| *codepoint);

	serialise(
		size,
		entries,
		glyph_table,
		settings.compress.unwrap_or(false),
	)
}

/// Concatenate encoded glyphs, returning the table and each glyph's byte offset.
/// Offset 0 means "glyph not supported", so the table opens with 4 pad bytes
fn layout(glyphs: &[EncodedGlyph]) -> (Vec<u8>, Vec<u32>) {
	let mut glyph_table = vec![0u8; 4];
	let mut offsets = Vec::with_capacity(glyphs.len());
	for glyph in glyphs {
		offsets.push(glyph_table.len() as u32);
		glyph_table.extend_from_slice(&glyph.header.to_bytes());
		glyph_table.extend_from_slice(&glyph.body);
	}
	(glyph_table, offsets)
}

fn bucket(entries: Vec<(u32, u32)>) -> Result<Vec<Vec<(u32, u32)>>> {
	let mut buckets: Vec<Vec<(u32, u32)>> = vec![Vec::new(); HASH_TABLE_SIZE];
	for (codepoint, offset) in entries {
		let bucket = &mut buckets[codepoint as usize % HASH_TABLE_SIZE];
		bucket.push((codepoint, offset));
		if bucket.len() > OFFSET_TABLE_MAX_SIZE {
			bail!(
				"Hash bucket for codepoint {codepoint:#x} exceeds {OFFSET_TABLE_MAX_SIZE} \
				 entries, subset the font further"
			);
		}
	}
	Ok(buckets)
}

fn serialise(
	size: u16,
	entries: Vec<(u32, u32)>,
	glyph_table: Vec<u8>,
	rle4: bool,
) -> Result<Vec<u8>> {
	let number_of_glyphs = entries.len();
	let codepoint_bytes: u8 = if entries.iter().any(|(cp, _)| *cp > 0xFFFF) {
		4
	} else {
		2
	};
	let mut features = if rle4 { FEATURE_RLE4 } else { 0 };
	let offset_bytes: u8 = if glyph_table.len() < 65536 {
		features |= FEATURE_OFFSET_16;
		2
	} else {
		4
	};
	let buckets = bucket(entries)?;

	let mut pbf = Vec::new();
	pbf.push(FONT_VERSION);
	pbf.push(u8::try_from(size).wrap_err("Font size over 255px")?);
	pbf.extend((number_of_glyphs as u16).to_le_bytes());
	pbf.extend((WILDCARD as u16).to_le_bytes());
	pbf.push(HASH_TABLE_SIZE as u8);
	pbf.push(codepoint_bytes);
	pbf.push(10);
	pbf.push(features);

	let entry_size = (codepoint_bytes + offset_bytes) as usize;
	let mut acc = 0usize;
	for (hash, bucket) in buckets.iter().enumerate() {
		pbf.push(hash as u8);
		pbf.push(bucket.len() as u8);
		pbf.extend(
			u16::try_from(acc)
				.wrap_err("Offset tables too large to index")?
				.to_le_bytes(),
		);
		acc += bucket.len() * entry_size;
	}
	for (codepoint, offset) in buckets.into_iter().flatten() {
		if codepoint_bytes == 2 {
			pbf.extend((codepoint as u16).to_le_bytes());
		} else {
			pbf.extend(codepoint.to_le_bytes());
		}
		if offset_bytes == 2 {
			pbf.extend((offset as u16).to_le_bytes());
		} else {
			pbf.extend(offset.to_le_bytes());
		}
	}
	pbf.extend(glyph_table);

	Ok(pbf)
}

fn get_font_size(settings: &FontSettings, name: &str) -> Option<u16> {
	settings.pixel_height.or(name_height(name))
}

fn name_height(name: &str) -> Option<u16> {
	let re = Regex::new("[0-9]+").expect("Valid regex");
	re.find(name)
		.map(|needle| needle.as_str().parse::<u16>().ok())
		.flatten()
}

/// Filter the font by Regex and alllowlist, Returning a list of (Codepoint, Index) and a list of
/// (Glyph index), deduplicating multiple codepoints that resolve to the same glyph
///
/// # Errors
///
/// This function will return an error if the regex or any codepoints are invalid, or the codepoints
/// list file is invalid or missing
fn get_subset(
	typeface: &freetype::Face,
	settings: &FontSettings,
	max_glyphs: usize,
) -> Result<(Vec<(char, usize)>, Vec<u32>)> {
	let mut subset = Vec::new();
	let mut index_map: BTreeMap<u32, usize> = BTreeMap::new();
	subset.push((WILDCARD, 0));
	index_map.insert(0, 0);
	let re = Regex::new(settings.character_regex.as_deref().unwrap_or(".*"))?;
	let list = load_char_list(settings.character_list.as_deref())?;
	let mut strbuf = [0u8; 4];

	let mut total_offset = 1;
	for (codepoint, gindex) in typeface.chars() {
		if subset.len() > max_glyphs {
			break;
		}
		let char = char::from_u32(codepoint as u32).ok_or_eyre("Invalid codepoint")?;
		// Wildcard and ellipsis are exempt from filtering. A font with its own
		// U+25AF gets a duplicate entry for it, matching upstream
		if char == WILDCARD
			|| char == ELLIPSIS
			|| (re.is_match(char.encode_utf8(&mut strbuf))
				&& list
					.as_ref()
					.map_or(char as u32 >= MIN_CODEPOINT, |l| l.contains(&(char as u32))))
		{
			let gindex = gindex.get();
			let index = match index_map.get(&gindex) {
				Some(i) => *i,
				None => {
					let index = total_offset;
					total_offset += 1;
					index_map.insert(gindex, index);
					index
				}
			};
			subset.push((char, index));
		}
	}

	// Pivot so gindex -> index becomes index_map[index] -> gindex
	let index_map: Vec<u32> = index_map
		.into_iter()
		.map(|(gindex, index)| (index, gindex))
		.collect::<BTreeMap<_, _>>()
		.into_values()
		.collect();

	Ok((subset, index_map))
}

fn load_char_list(path: Option<&Path>) -> Result<Option<HashSet<u32>>> {
	let Some(path) = path else {
		return Ok(None);
	};
	let list: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
	let Some(Value::Array(ar)) = list.get("codepoints") else {
		bail!("No codepoints list in file")
	};
	let set = ar
		.iter()
		.filter_map(|v| v.as_u64().and_then(|num| num.try_into().ok()))
		.collect();
	Ok(Some(set))
}

fn encode_glyph(
	typeface: &freetype::Face,
	i: u32,
	settings: &FontSettings,
	baseline: u16,
	max_glyph_size: usize,
) -> Result<EncodedGlyph> {
	typeface.load_glyph(
		i,
		LoadFlag::RENDER | LoadFlag::MONOCHROME | LoadFlag::TARGET_MONO,
	)?;
	let glyph = typeface.glyph();
	let bitmap = glyph.bitmap();
	// Outline glyphs always rasterise 1bpp under these load flags, but embedded
	// bitmap strikes keep their native mode. Legacy (`compatibility`) fonts, which
	// upstream renders 8bpp grey and thresholds at >127, are also unsupported for now
	let mode = bitmap.pixel_mode()?;
	if !matches!(mode, freetype::bitmap::PixelMode::Mono) {
		bail!("Unsupported pixel mode {mode:?}, expected 1bpp monochrome");
	}
	let mut header = GlyphHeader {
		width:       u8::try_from(bitmap.width()).wrap_err_with(|| format!("Glyph {i} width"))?,
		height:      u8::try_from(bitmap.rows()).wrap_err_with(|| format!("Glyph {i} height"))?,
		offset_left: i8::try_from(glyph.bitmap_left())
			.wrap_err_with(|| format!("Glyph {i} left offset"))?,
		offset_top:  i8::try_from(baseline as i32 - glyph.bitmap_top())
			.wrap_err_with(|| format!("Glyph {i} top offset"))?,
		advance:     i8::try_from(
			(glyph.advance().x >> 6) + settings.tracking_adjust.unwrap_or(0) as i64,
		)
		.wrap_err_with(|| format!("Glyph {i} advance"))?,
	};

	let body = if header.width == 0 || header.height == 0 {
		// Empty when char is 0 width or height
		vec![]
	} else {
		let buf = bitmap.buffer();
		let pitch = bitmap.pitch() as usize;
		let (width, height) = (header.width as usize, header.height as usize);
		// Source rows are pitch bytes wide with bits MSB-first
		let bits = (0..height)
			.flat_map(|row| (0..width).map(move |col| (row * pitch + col / 8, 0x80 >> (col % 8))))
			.map(|(byte, mask)| buf[byte] & mask != 0);

		if settings.compress.unwrap_or(false) {
			let (packed, units) = compress_rle4(bits);
			// The header's height field is repurposed as the RLE unit count
			header.height = u8::try_from(units)
				.wrap_err_with(|| format!("Glyph {i} needs more than 255 RLE4 units"))?;
			check_rle4_decodes_in_place(&packed, units, max_glyph_size)
				.wrap_err_with(|| format!("Glyph {i}"))?;
			packed
		} else {
			let size_bytes = (width * height).div_ceil(8);
			if size_bytes > max_glyph_size {
				bail!("Glyph {i} too large: {size_bytes} > {max_glyph_size} bytes");
			}
			pack_lsb_words(bits, width * height)
		}
	};

	Ok(EncodedGlyph { header, body })
}

/// Pack pixels row-major LSB-first into little-endian u32 words
fn pack_lsb_words(bits: impl Iterator<Item = bool>, len: usize) -> Vec<u8> {
	let mut words = vec![0u32; len.div_ceil(32)];
	for (pixel, bit) in bits.enumerate() {
		if bit {
			words[pixel / 32] |= 1 << (pixel % 32);
		}
	}
	words.into_iter().flat_map(u32::to_le_bytes).collect()
}

/// Run-length encode into 4-bit units (colour << 3 | run - 1, runs of 1-8), two
/// units per byte low-nibble first, padded to 4 bytes. Returns the unpadded
/// unit count
fn compress_rle4(bits: impl Iterator<Item = bool>) -> (Vec<u8>, usize) {
	let mut units: Vec<(bool, u8)> = Vec::new();
	for bit in bits {
		match units.last_mut() {
			Some((colour, run)) if *colour == bit && *run < 8 => *run += 1,
			_ => units.push((bit, 1)),
		}
	}
	let num_units = units.len();
	if num_units % 2 == 1 {
		units.push((false, 1));
	}
	let mut packed: Vec<u8> = units
		.chunks(2)
		.map(|pair| {
			let ((c1, r1), (c2, r2)) = (pair[0], pair[1]);
			(c1 as u8) << 3 | (r1 - 1) | (c2 as u8) << 7 | (r2 - 1) << 4
		})
		.collect();
	while packed.len() % 4 != 0 {
		packed.push(0);
	}
	(packed, num_units)
}

/// The firmware decodes RLE4 in place: the encoded glyph sits at the end of a
/// max_glyph_size buffer and decoded bytes are written from the front (past the
/// header). Simulate that to prove the write pointer never catches the read
/// pointer
fn check_rle4_decodes_in_place(
	packed: &[u8],
	mut units: usize,
	max_glyph_size: usize,
) -> Result<()> {
	let mut dst = size_of::<GlyphHeader>();
	let base = max_glyph_size
		.checked_sub(packed.len())
		.ok_or_eyre("RLE4 stream larger than the glyph buffer")?;
	let mut src = base;
	let mut out_bits = 0usize;
	while units > 0 {
		if src >= max_glyph_size {
			bail!("RLE4 stream larger than the glyph buffer");
		}
		let mut pair = packed[src - base];
		src += 1;
		for _ in 0..units.min(2) {
			out_bits += (pair as usize & 0x07) + 1;
			if out_bits >= 8 {
				if dst >= src {
					bail!("Cannot RLE4 decode in place, decode overruns the stream");
				}
				if dst >= max_glyph_size {
					bail!("Decoded bitmap larger than the glyph buffer");
				}
				dst += 1;
				out_bits -= 8;
			}
			pair >>= 4;
			units -= 1;
		}
	}
	Ok(())
}

pub(crate) fn check_font(file: &Path) -> Result<Vec<u8>> {
	check_font_bytes(std::fs::read(file)?)
}

fn check_font_bytes(pbf: Vec<u8>) -> Result<Vec<u8>> {
	if pbf.len() < 8 {
		bail!("PBF smaller than its FontInfo header ({} bytes)", pbf.len());
	}
	let version = pbf[0];
	// FontInfo v2 is 8 bytes with fixed 4-byte offsets; v3 adds struct size and features
	let (header_len, offset_bytes) = match version {
		2 => (8, 4),
		3 => {
			let (size, features) = (
				*pbf.get(8).ok_or_eyre("PBF smaller than a v3 FontInfo")?,
				*pbf.get(9).ok_or_eyre("PBF smaller than a v3 FontInfo")?,
			);
			if size < 10 {
				bail!("v3 FontInfo size {size} < 10");
			}
			if features & !(FEATURE_OFFSET_16 | FEATURE_RLE4) != 0 {
				bail!("Unknown feature bits in {features:#04x}");
			}
			let offset_bytes = if features & FEATURE_OFFSET_16 != 0 {
				2
			} else {
				4
			};
			(size as usize, offset_bytes)
		}
		v => bail!("Unsupported PBF version {v}"),
	};
	let max_height = pbf[1];
	let number_of_glyphs = u16::from_le_bytes([pbf[2], pbf[3]]) as usize;
	let hash_table_size = pbf[6] as usize;
	let codepoint_bytes = pbf[7] as usize;
	if max_height == 0 {
		bail!("PBF max_height is 0");
	}
	if hash_table_size == 0 {
		bail!("PBF hash table size is 0");
	}
	if !matches!(codepoint_bytes, 2 | 4) {
		bail!("PBF codepoint_bytes is {codepoint_bytes}, expected 2 or 4");
	}

	let entry_size = codepoint_bytes + offset_bytes;
	let hash_end = header_len + hash_table_size * 4;
	let tables_len = number_of_glyphs * entry_size;
	// The glyph table is data-dependent but holds at least its 4-byte pad word
	let min_len = hash_end + tables_len + 4;
	if pbf.len() < min_len {
		bail!(
			"PBF is {} bytes; FontInfo + {hash_table_size} hash records + {number_of_glyphs} \
			 entries need at least {min_len}",
			pbf.len()
		);
	}

	let mut acc = 0usize;
	for (i, record) in pbf[header_len..hash_end].chunks_exact(4).enumerate() {
		if record[0] as usize != i {
			bail!("Hash record {i} is labelled {}", record[0]);
		}
		let offset = u16::from_le_bytes([record[2], record[3]]) as usize;
		if offset != acc {
			bail!("Hash record {i} points at byte {offset}, expected {acc}");
		}
		acc += record[1] as usize * entry_size;
	}
	if acc != tables_len {
		bail!(
			"Hash buckets cover {} entries, FontInfo declares {number_of_glyphs}",
			acc / entry_size
		);
	}

	Ok(pbf)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Validate an existing .pbf via check_font. Set PBF_CHECK to the file path;
	/// skipped when unset
	#[test]
	fn check_existing_pbf() -> Result<()> {
		let Ok(file) = std::env::var("PBF_CHECK") else {
			return Ok(());
		};
		check_font(Path::new(&file)).map(drop)
	}

	/// Byte-parity against upstream fontgen.py. Set PBF_PARITY_TTF, PBF_PARITY_REF
	/// (reference .pbf), PBF_PARITY_NAME, and PBF_PARITY_SETTINGS (FontSettings json);
	/// skipped when unset
	#[test]
	fn parity_with_upstream_fontgen() -> Result<()> {
		let Ok(ttf) = std::env::var("PBF_PARITY_TTF") else {
			return Ok(());
		};
		let reference = std::fs::read(std::env::var("PBF_PARITY_REF")?)?;
		let settings: FontSettings = serde_json::from_str(&std::env::var("PBF_PARITY_SETTINGS")?)?;
		let name = std::env::var("PBF_PARITY_NAME")?;
		let ours = check_font_bytes(process_font(
			Path::new(&ttf),
			&name,
			Platform::Emery,
			settings,
		)?)?;
		if let Ok(dump) = std::env::var("PBF_PARITY_DUMP") {
			std::fs::write(dump, &ours)?;
		}
		let diff = ours.iter().zip(&reference).position(|(a, b)| a != b);
		assert_eq!(diff, None, "first differing byte");
		assert_eq!(ours.len(), reference.len(), "length mismatch");
		Ok(())
	}
}
