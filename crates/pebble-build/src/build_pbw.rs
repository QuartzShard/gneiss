use core::mem::size_of;
use std::collections::BTreeMap;

use color_eyre::eyre::{Context, OptionExt, Result, bail, eyre};
use object::{Object, ObjectSection, ObjectSymbol, SectionKind, elf::SHF_ALLOC};
use pebble_types::{
	Platform,
	header::{PebbleProcessInfo, Version},
	manifest::{PebbleAppInfo, PebbleAppManifest},
	pbpack::PbPack,
	stm_crc32::crc,
};
use unix_time::Instant;

use crate::{MessageKey, PebbleMetadata};

fn find_symbol(elf: &object::File<'_>, name: &str) -> Result<u32> {
	elf.symbols()
		.find(|sym| sym.name() == Ok(name))
		.map(|sym| sym.address() as u32)
		.ok_or_else(|| eyre!("Cannot locate symbol: {}", name))
}

fn write_cstr_field(field: &mut [std::ffi::c_char], name: &str) -> Result<()> {
	let len = name.len();
	if len >= field.len() {
		bail!(
			"{name:?} is {} bytes too long for the buffer",
			name.len() - field.len()
		)
	}
	for (c, b) in field.iter_mut().zip(name.as_bytes()) {
		*c = *b as std::ffi::c_char;
	}
	Ok(())
}

fn read_elf(elf_file: &[u8]) -> Result<(object::File<'_>, Vec<u32>)> {
	let elf: object::File<'_> = object::File::parse(elf_file)?;
	for section in elf.sections() {
		log::debug!("{:?}", section.name().unwrap_or(""));
	}
	let got = elf
		.sections()
		.find(|s| s.name() == Ok(".got"))
		.into_iter()
		.flat_map(|got| {
			(got.address()..(got.address() + got.size()))
				.step_by(4)
				.map(|addr| addr as u32)
		});
	let relocations: Vec<u32> = elf
		.sections()
		.filter(is_loadable)
		.flat_map(|s| s.relocations())
		.filter_map(|(offset, rel)| match rel.kind() {
			object::RelocationKind::Absolute => Some(offset as u32),
			_ => None,
		})
		.chain(got)
		.collect();
	log::debug!("Relocations: {relocations:?}");
	Ok((elf, relocations))
}

fn objcopy(elf: &object::File<'_>) -> Result<(usize, Vec<u8>)> {
	let loadable: Vec<_> = elf.sections().filter(is_loadable).collect();
	let bin_size = loadable
		.iter()
		.map(|sec| sec.address() + sec.size())
		.max()
		.unwrap_or(0) as usize;
	let mut bin_buf = vec![0u8; bin_size];
	for section in loadable {
		// Address is offset because:
		// APP (rwx) : ORIGIN = 0, LENGTH = @MAX_APP_MEMORY_SIZE@
		let offset = section.address() as usize;
		bin_buf[offset..offset + section.data()?.len()].copy_from_slice(section.data()?);
	}
	Ok((bin_size, bin_buf))
}

fn is_loadable(s: &object::Section<'_, '_>) -> bool {
	let alloc = matches!(s.flags(), object::SectionFlags::Elf { sh_flags } if sh_flags & u64::from(SHF_ALLOC) != 0);
	let bss = matches!(s.kind(), SectionKind::UninitializedData);
	alloc && !bss
}

#[allow(clippy::too_many_arguments)]
fn build_pbl_header(
	pkg: &cargo_metadata::Package,
	elf: &object::File<'_>,
	pbl_metadata: &PebbleMetadata,
	platform: Platform,
	pbpack: &PbPack,
	relocations: &[u32],
	timestamp: Instant,
	bin_size: usize,
	bin_buf: &[u8],
) -> Result<PebbleProcessInfo> {
	let pbl_header_section = elf
		.sections()
		.find(|section| section.name().unwrap_or("") == ".header")
		.ok_or_eyre(".pbl_header missing")?;
	let mut pbl_header: PebbleProcessInfo =
		bytemuck::pod_read_unaligned(&pbl_header_section.data()?[..size_of::<PebbleProcessInfo>()]);
	let sentinel: [std::ffi::c_char; 8] = unsafe { std::mem::transmute(*b"PBLAPP\0\0") };
	if pbl_header.header != sentinel {
		bail!("Header sentinel malformed")
	}
	log::debug!("Found header, populating...");
	pbl_header.sdk_version = platform.config().sdk_version;
	pbl_header.process_version = Version {
		major: pkg.version.major as u8,
		minor: pkg.version.minor as u8,
	};
	pbl_header.load_size = bin_size.try_into().wrap_err("Image too large!")?;
	pbl_header.offset = find_symbol(elf, "main")? & !1;
	pbl_header.crc = crc(&bin_buf[size_of::<PebbleProcessInfo>()..]);
	write_cstr_field(&mut pbl_header.name, &pkg.name)?;
	write_cstr_field(
		&mut pbl_header.company,
		pkg.authors.first().map_or("anonymous", String::as_str),
	)?;
	pbl_header.icon_resource_id = crate::icon_index(pbl_metadata, platform)?.unwrap_or(0);
	pbl_header.sym_table_addr = find_symbol(elf, "pbl_table_addr")?;
	pbl_header.flags = u32::from(pbl_metadata.watchface)
		| u32::from(platform.config().process_info_platform_bit) << 6;
	pbl_header.num_reloc_entries = relocations.len() as u32;
	pbl_header.uuid = pbl_metadata.uuid.as_bytes()[..]
		.try_into()
		.wrap_err("UUID array len wrong")?;
	pbl_header.resource_crc = pbpack.manifest().crc();
	pbl_header.resource_timestamp = timestamp.secs() as u32;
	pbl_header.virtual_size = elf
		.sections()
		.filter(|s| {
			matches!(s.flags(), object::SectionFlags::Elf { sh_flags } if sh_flags & u64::from(SHF_ALLOC) != 0)
		})
		.map(|sec| sec.address() + sec.size())
		.max()
		.unwrap_or(0)
		.try_into()
		.wrap_err("Virtual Size too large!")?;
	log::debug!("{pbl_header:?}");
	Ok(pbl_header)
}

/// Package one platform's compiled ELF into its shipped `.bin` (header patched,
/// reloc table appended) and the matching `manifest.json` string.
pub fn package_platform(
	elf_bytes: &[u8],
	pkg: &cargo_metadata::Package,
	pbl_metadata: &PebbleMetadata,
	platform: Platform,
	pbpack: &PbPack,
	timestamp: Instant,
) -> Result<(Vec<u8>, String, u16)> {
	let (elf, relocations) = read_elf(elf_bytes)?;
	let (bin_size, mut bin_buf) = objcopy(&elf)?;
	let pbl_header = build_pbl_header(
		pkg,
		&elf,
		pbl_metadata,
		platform,
		pbpack,
		&relocations,
		timestamp,
		bin_size,
		&bin_buf,
	)?;

	bin_buf[..size_of::<PebbleProcessInfo>()].copy_from_slice(bytemuck::bytes_of(&pbl_header));
	bin_buf.reserve(relocations.len() * 4);
	bin_buf.extend(relocations.iter().flat_map(|word| word.to_le_bytes()));

	let config = platform.config();
	if bin_buf.len() > config.max_app_binary_size as usize {
		bail!(
			"App binary is {} bytes, over the {} {} byte limit",
			bin_buf.len(),
			platform,
			config.max_app_binary_size
		);
	}
	let pbpack_bytes = pbpack.as_bytes();
	if pbpack_bytes.len() > config.max_resources_size as usize {
		bail!(
			"Resource pack is {} bytes, over the {} {} byte limit",
			pbpack_bytes.len(),
			platform,
			config.max_resources_size
		);
	}
	let manifest = serde_json::to_string_pretty(&PebbleAppManifest::new(
		bin_buf.len() as u64,
		crc(&bin_buf),
		pbpack_bytes.len() as u64,
		crc(&pbpack_bytes),
		timestamp,
	))?;
	Ok((bin_buf, manifest, pbl_header.virtual_size))
}

/// Root `appinfo.json` for the bundle — app identity + the list of bundled
/// target platforms.
pub fn app_info(
	pkg: &cargo_metadata::Package,
	pbl_metadata: &mut PebbleMetadata,
	platforms: &[Platform],
) -> Result<String> {
	let package_semver = pkg.version.to_string();
	let message_keys = resolve_message_keys(pbl_metadata.message_keys.as_deref_mut());
	Ok(serde_json::to_string_pretty(&PebbleAppInfo::new(
		pbl_metadata.uuid,
		&pkg.name,
		pkg.authors.first().map_or("anonymous", String::as_str),
		&package_semver,
		platforms,
		pbl_metadata.watchface,
		message_keys.clone(),
		message_keys,
	))?)
}

/// Resolved message keys as the pretty `name -> u32` JSON the companion reads.
/// Empty when no keys are declared; callers skip writing in that case.
pub fn message_keys_json(keys: Option<&mut [MessageKey]>) -> Result<String> {
	Ok(serde_json::to_string_pretty(&resolve_message_keys(keys))?)
}

pub fn resolve_message_keys(keys: Option<&mut [MessageKey]>) -> BTreeMap<&str, u32> {
	let mut map = BTreeMap::new();
	// Array sorts lower than Single, so this produces Arrays-First, then lex
	if let Some(keys) = keys {
		keys.sort();
		keys.iter().fold(0, |acc, key| {
			map.insert(key.name(), acc);
			acc + key.len()
		});
	}
	map
}
