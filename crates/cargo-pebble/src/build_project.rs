use std::{
	fs::create_dir_all,
	io::{BufReader, Write},
	process::{Command, Stdio, exit},
};

use cargo_metadata::{Message, PackageId, camino::Utf8PathBuf};
use color_eyre::eyre::{OptionExt, Result};
use pebble_build::{app_info, message_keys_json, package_platform, read_pbl_metadata};
use pebble_types::{Platform, pbpack::PbPack};
use unix_time::Instant;
use zip::write::FileOptions;

fn build_elf(
	platform: Platform,
	pkg_id: &PackageId,
) -> Result<(Option<Utf8PathBuf>, Option<Utf8PathBuf>)> {
	let mut build = Command::new("cargo")
		.arg("build")
		.arg("--release")
		// Requested platform only; ignore a default-platform feature (which exists
		// so a bare `cargo check`/rust-analyzer can build).
		.arg("--no-default-features")
		.arg(format!("--features={platform}"))
		.arg("--message-format=json-render-diagnostics")
		.stdout(Stdio::piped())
		.stderr(Stdio::inherit())
		.spawn()?;
	let stdout = build.stdout.take().expect("piped");
	let mut elf: Option<Utf8PathBuf> = None;
	let mut out_dir: Option<Utf8PathBuf> = None;
	let mut build_ok = false;
	for msg in Message::parse_stream(BufReader::new(stdout)) {
		match msg? {
			Message::CompilerArtifact(artifact) if artifact.package_id == *pkg_id => {
				log::trace!("{artifact:#?}");
				elf = artifact.executable;
			}
			Message::BuildScriptExecuted(build) if build.package_id == *pkg_id => {
				log::trace!("{build:#?}");
				out_dir.replace(build.out_dir);
			}
			Message::BuildFinished(build_finished) => build_ok = build_finished.success,
			_ => (),
		}
	}
	if !build.wait()?.success() || !build_ok {
		// `cargo build` already streamed its own error to our inherited stderr;
		// piling a redundant top-level report on top just adds noise.
		exit(1);
	}
	Ok((elf, out_dir))
}

fn bytes_readable(val: usize) -> String {
	match val {
		1_000_000.. => format!("{}.{}MB", val / 1_000_000, (val % 1_000_000) / 1000),
		1000..1_000_000 => format!("{}.{}KB", val / 1000, val % 1000),
		_ => format!("{val} B"),
	}
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

		let (cargo_meta, pkg, mut pbl_metadata) = read_pbl_metadata()?;
		let triple = self.platforms[0].app_target_triple();
		let build_path = cargo_meta.target_directory.join(triple).join("release");
		create_dir_all(&build_path)?;

		let appinfo_json = app_info(&pkg, &mut pbl_metadata, &self.platforms)?;

		if pbl_metadata
			.message_keys
			.as_deref()
			.is_some_and(|k| !k.is_empty())
		{
			let json = message_keys_json(pbl_metadata.message_keys.as_deref_mut())?;
			std::fs::write(build_path.join("message_keys.json"), json)?;
		}

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
		pbw.write_all(appinfo_json.as_bytes())?;

		for &platform in &self.platforms {
			log::info!("Building pebble-app.bin for {platform}");
			let (elf_path, out_dir) = build_elf(platform, &pkg.id)?;
			let elf_path =
				elf_path.ok_or_eyre("No executable - make sure you're building a [[bin]]")?;
			let out_dir = out_dir.ok_or_eyre("No out dir")?;
			log::debug!("ELF output at {elf_path}");
			let elf_bytes = std::fs::read(&elf_path)?;

			let pbpack = PbPack::from_bytes(&std::fs::read(
				out_dir.join(format!("{platform}_app_resources.pbpack")),
			)?)
			.ok_or_eyre("PbPack Malformed")?;
			let pbpack_bytes = pbpack.as_bytes();

			let (bin_buf, manifest, virt_size) = package_platform(
				&elf_bytes,
				&pkg,
				&pbl_metadata,
				platform,
				&pbpack,
				timestamp,
			)?;

			pbw.start_file(format!("{platform}/pebble-app.bin"), FileOptions::default())?;
			pbw.write_all(&bin_buf)?;
			pbw.start_file(format!("{platform}/manifest.json"), FileOptions::default())?;
			pbw.write_all(manifest.as_bytes())?;
			pbw.start_file(
				format!("{platform}/app_resources.pbpack"),
				FileOptions::default(),
			)?;
			pbw.write_all(&pbpack_bytes)?;

			log::info!(
				"{platform} space usge:\n\
                    -> RAM (static): {} / {}, {} left for heap\n\
                    -> Flash: {} binary, {} resources",
				bytes_readable(virt_size as usize),
				bytes_readable(platform.config().max_app_memory_size as usize),
				bytes_readable(
					(platform.config().max_app_memory_size - u32::from(virt_size)) as usize
				),
				bytes_readable(bin_buf.len()),
				bytes_readable(pbpack_bytes.len()),
			);
		}
		pbw.finish()?;
		log::info!("Success! App bundle written at {bundle_path}");
		Ok(())
	}
}
