use core::mem::size_of;
use std::{
	collections::BTreeMap,
	io::{BufReader, Write},
	process::{Command, Stdio},
};

use cargo_metadata::{Message, MetadataCommand, PackageId, camino::Utf8PathBuf};
use color_eyre::eyre::{Context, OptionExt, Result, bail, eyre};
use object::{Object, ObjectSection, ObjectSymbol, SectionKind, elf::SHF_ALLOC};
use pebble_types::{
	Platform,
	header::{PebbleProcessInfo, Version},
	manifest::{PebbleAppInfo, PebbleAppManifest},
	pbpack::PbPack,
	stm_crc32::crc,
};
use serde::Deserialize;
use unix_time::Instant;
use zip::write::FileOptions;

use crate::template::PebbleMetadata;

fn find_symbol(elf: &object::File<'_>, name: &str) -> Result<u32> {
	elf.symbols()
		.find(|sym| sym.name() == Ok(name))
		.map(|sym| sym.address() as u32)
		.ok_or_else(|| eyre!("Cannot locate symbol: {}", name))
}

fn write_cstr_field(field: &mut [std::ffi::c_char], name: &str) -> Result<()> {
	let len = name.as_bytes().len();
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

fn build_elf(
	platform: Platform,
	pkg_id: &PackageId,
) -> Result<Option<Utf8PathBuf>, color_eyre::eyre::Error> {
	let features = format!("--features={platform}");
	let mut build = Command::new("cargo")
		.arg("build")
		.arg("--release")
		.arg(features)
		.arg("--message-format=json-render-diagnostics")
		.stdout(Stdio::piped())
		.stderr(Stdio::inherit())
		.spawn()?;
	let stdout = build.stdout.take().expect("piped");
	let mut elf: Option<Utf8PathBuf> = None;
	let mut build_ok = false;
	for msg in Message::parse_stream(BufReader::new(stdout)) {
		match msg? {
			Message::CompilerArtifact(artifact) if artifact.package_id == *pkg_id => {
				log::trace!("{:#?}", artifact);
				elf = artifact.executable;
			}
			Message::BuildFinished(build_finished) => build_ok = build_finished.success,
			_ => (),
		};
	}
	if !build.wait()?.success() || !build_ok {
		bail!("cargo build failed")
	}
	Ok(elf)
}
fn read_elf(elf_file: &[u8]) -> Result<(object::File<'_>, Vec<u32>), color_eyre::eyre::Error> {
	let elf: object::File<'_> = object::File::parse(&*elf_file)?;
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
		.find(|s| s.name() == Ok(".data"))
		.into_iter()
		.flat_map(|s| s.relocations())
		.map(|rel| rel.0 as u32)
		.chain(got)
		.collect();
	log::debug!("Relocations: {relocations:?}");
	Ok((elf, relocations))
}
fn objcopy(elf: &object::File<'_>) -> Result<(usize, Vec<u8>), color_eyre::eyre::Error> {
	let loadable: Vec<_> = elf
			    .sections()
			    .filter(|s| {
                let alloc =	matches!(s.flags(), object::SectionFlags::Elf { sh_flags } if sh_flags & SHF_ALLOC as u64 != 0);
                let bss = matches!(s.kind(), SectionKind::UninitializedData);
                alloc && !bss
			    })
			    .collect();
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

fn build_pbl_header(
	pkg: &cargo_metadata::Package,
	elf: object::File<'_>,
	pbl_metadata: &PebbleMetadata,
	platform: Platform,
	pbpack: &PbPack,
	relocations: &Vec<u32>,
	timestamp: Instant,
	bin_size: usize,
	bin_buf: &Vec<u8>,
) -> Result<PebbleProcessInfo, color_eyre::eyre::Error> {
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
	pbl_header.process_version = Version {
		major: pkg.version.major as u8,
		minor: pkg.version.minor as u8,
	};
	pbl_header.load_size = bin_size.try_into().wrap_err("Image too large!")?;
	pbl_header.offset = find_symbol(&elf, "main")?;
	pbl_header.crc = crc(&bin_buf[size_of::<PebbleProcessInfo>()..]);
	write_cstr_field(&mut pbl_header.name, &pkg.name)?;
	write_cstr_field(
		&mut pbl_header.company,
		&pkg.authors
			.first()
			.map(String::as_str)
			.unwrap_or("anonymous"),
	)?;
	// TODO: Resources
	pbl_header.icon_resource_id = 0;
	pbl_header.sym_table_addr = find_symbol(&elf, "pbl_table_addr")?;
	pbl_header.flags =
		pbl_metadata.watchface as u32 | (platform.config().process_info_platform_bit as u32) << 6;
	pbl_header.num_reloc_entries = relocations.len() as u32;
	pbl_header.uuid = pbl_metadata.uuid.as_bytes()[..]
		.try_into()
		.wrap_err("UUID array len wrong")?;
	pbl_header.resource_crc = pbpack.manifest().crc();
	pbl_header.resource_timestamp = timestamp.secs() as u32;
	pbl_header.virtual_size = elf
        .sections()
        .filter(|s| {
            matches!(s.flags(), object::SectionFlags::Elf { sh_flags } if sh_flags & SHF_ALLOC as u64 != 0)
		}).map(|sec| sec.address() + sec.size()).max().unwrap_or(0).try_into().wrap_err("Virtual Size too large!")?;
	log::debug!("{pbl_header:?}");
	Ok(pbl_header)
}

#[derive(Debug, Clone, clap::Parser)]
/// Build your Pebble app project
pub struct BuildProject {
	platforms: Vec<Platform>,
}

impl BuildProject {
	pub fn build(&self) -> Result<()> {
		log::info!("Starting build...");
		let timestamp = Instant::now();

		let cargo_meta = MetadataCommand::new().exec()?;
		let pkg = cargo_meta
			.root_package()
			.ok_or_eyre("No root package")?
			.clone();
		let pbl_metadata = PebbleMetadata::deserialize(
			pkg.metadata
				.get("pebble")
				.ok_or_eyre("No pebble manifest")?,
		)
		.wrap_err("Malformed pebble manifest")?;

		let pbpack = PbPack::new(&[]);
		let pbpack_bytes = pbpack.as_bytes();

		let package_semver = pkg.version.to_string();
		let app_info = serde_json::to_string_pretty(&PebbleAppInfo::new(
			pbl_metadata.uuid,
			&pkg.name,
			pkg.authors
				.get(0)
				.map(String::as_str)
				.unwrap_or("anonymous"),
			&package_semver,
			&self.platforms,
			pbl_metadata.watchface,
			// TODO: appKey and messageKeys
			BTreeMap::new(),
			BTreeMap::new(),
		))?;
		let triple = &self.platforms[0].app_target_triple();
		let build_path = cargo_meta.target_directory.join(triple).join("release");
		println!("{build_path:?}");
		let bundle_path = build_path.join(format!("{}.pbw", pkg.name));
		let mut out_file = std::io::BufWriter::new(
			std::fs::OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(&bundle_path)?,
		);
		let mut pbw = zip::write::ZipWriter::new(&mut out_file);
		pbw.start_file("appinfo.json", FileOptions::default())?;
		pbw.write_all(app_info.as_bytes())?;
		for platform in &self.platforms {
			let platform = *platform;
			log::info!("Building pebble-app.bin for {}", platform);
			let elf = build_elf(platform, &pkg.id)?;

			let elf_path = elf.ok_or_eyre("No executable - make sure you're building a [[bin]]")?;
			log::debug!("ELF output at {elf_path}");
			let elf_file = std::fs::read(&elf_path)?;
			let (elf, relocations) = read_elf(&elf_file)?;

			let (bin_size, mut bin_buf) = objcopy(&elf)?;

			let pbl_header = build_pbl_header(
				&pkg,
				elf,
				&pbl_metadata,
				platform,
				&pbpack,
				&relocations,
				timestamp,
				bin_size,
				&bin_buf,
			)?;

			bin_buf[..size_of::<PebbleProcessInfo>()]
				.copy_from_slice(bytemuck::bytes_of(&pbl_header));
			bin_buf.reserve(relocations.len() * 4);
			bin_buf.extend(relocations.iter().flat_map(|word| word.to_le_bytes()));

			// Manifest
			let manifest = serde_json::to_string_pretty(&PebbleAppManifest::new(
				bin_buf.len() as u64,
				crc(&bin_buf),
				pbpack_bytes.len() as u64,
				crc(&pbpack_bytes),
				timestamp,
			))?;

			pbw.start_file(format!("{platform}/pebble-app.bin"), FileOptions::default())?;
			pbw.write_all(&bin_buf)?;
			pbw.start_file(format!("{platform}/manifest.json"), FileOptions::default())?;
			pbw.write_all(manifest.as_bytes())?;
			pbw.start_file(
				format!("{platform}/app_resources.pbpack"),
				FileOptions::default(),
			)?;
			pbw.write_all(&pbpack_bytes)?;
		}
		pbw.finish()?;
		log::info!("Success! App bundle written at {}", bundle_path);
		Ok(())
	}
}
