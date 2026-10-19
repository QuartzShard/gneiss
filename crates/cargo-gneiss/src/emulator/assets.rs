//! The emulator and its firmware, downloaded once into [`cache_dir`]:
//!
//! - QEMU: Core's build, from a `coredevices/qemu` release
//! - emery, flint, gabbro: the SDK shell firmware from the PebbleOS release the vendored headers
//!   came from, [`FIRMWARE_RELEASE`]
//! - aplite, basalt, chalk, diorite: frozen, so the firmware from app SDK [`FROZEN_SDK`]
//!
//! What's missing is listed and confirmed before anything is downloaded. GitHub downloads are
//! checked against their release's SHA-256 digests, and the SDK against [`FROZEN_SDK_SHA256`].
use std::{
	fmt::Write as _,
	io::{IsTerminal, Read, Write as _},
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail, eyre},
};
use gneiss_types::{FIRMWARE_RELEASE, Platform};
use sha2::{Digest, Sha256};

const QEMU_RELEASE: &str = "v10.1.5-pebble19";
const FROZEN_SDK: &str = "4.33.1";
const FROZEN_SDK_URL: &str = "https://sdk.repebble.com/releases/4.33.1/sdk-core.tar.gz";
const FROZEN_SDK_SHA256: &str = "a4d8a64f9a82b5e86006c5042852c9921461519779c31a45b0483a5d1ce8e875";
const FROZEN_SDK_SIZE: u64 = 48_319_014;
const MICRO_FLASH: &str = "qemu_micro_flash.bin";
const SPI_FLASH: &str = "qemu_spi_flash.bin";
/// Larger than any file we fetch
const DOWNLOAD_LIMIT: u64 = 256 << 20;

pub struct Assets {
	pub qemu:        PathBuf,
	/// The firmware QEMU boots
	pub micro_flash: PathBuf,
	/// A fresh SPI flash image
	pub spi_flash:   PathBuf,
	/// The firmware's version, which a flash image belongs to
	pub version:     String,
}

/// A file to download, and what becomes of it.
struct Download {
	url:    String,
	size:   u64,
	sha256: String,
	unpack: Unpack,
}

enum Unpack {
	/// A QEMU release archive, unpacked into this directory
	Qemu(PathBuf),
	/// A file kept as it is
	File(PathBuf),
	/// The app SDK, whose frozen platforms' firmware goes under this directory
	FrozenSdk(PathBuf),
}

/// The firmware version `platform`'s emulator runs.
pub fn version(platform: Platform) -> &'static str {
	if platform.config().frozen_at_revision.is_some() {
		FROZEN_SDK
	} else {
		FIRMWARE_RELEASE
	}
}

/// Everything `platform`'s emulator needs, downloading what isn't cached. Asks first, unless `yes`.
pub fn locate(platform: Platform, yes: bool) -> Result<Assets> {
	let cache = cache_dir()?;
	let version = version(platform);
	let dir = cache.join("firmware").join(version).join(platform.as_str());
	let downloads = missing(platform, &cache, &dir)?;
	if !downloads.is_empty() {
		confirm(platform, &downloads, yes)?;
		for download in &downloads {
			fetch(download)?;
		}
	}
	Ok(Assets {
		qemu:        qemu_dir(&cache).join("bin/qemu-pebble"),
		micro_flash: dir.join(MICRO_FLASH),
		spi_flash:   dir.join(SPI_FLASH),
		version:     version.to_owned(),
	})
}

fn qemu_dir(cache: &Path) -> PathBuf {
	cache.join("qemu").join(QEMU_RELEASE)
}

/// What `platform`'s emulator needs that isn't in `cache`, its firmware going in `dir`.
fn missing(platform: Platform, cache: &Path, dir: &Path) -> Result<Vec<Download>> {
	let mut downloads = Vec::new();
	let qemu_dir = qemu_dir(cache);
	if !qemu_dir.join("bin/qemu-pebble").exists() {
		let (url, size, sha256) = github_asset("coredevices/qemu", QEMU_RELEASE, &qemu_archive()?)?;
		downloads.push(Download {
			url,
			size,
			sha256,
			unpack: Unpack::Qemu(qemu_dir),
		});
	}
	let firmware = [("micro_flash", MICRO_FLASH), ("spi_flash", SPI_FLASH)];
	let absent: Vec<_> = firmware
		.into_iter()
		.filter(|(_, file)| !dir.join(file).exists())
		.collect();
	if absent.is_empty() {
		return Ok(downloads);
	}
	if platform.config().frozen_at_revision.is_some() {
		downloads.push(Download {
			url:    FROZEN_SDK_URL.to_owned(),
			size:   FROZEN_SDK_SIZE,
			sha256: FROZEN_SDK_SHA256.to_owned(),
			unpack: Unpack::FrozenSdk(cache.join("firmware").join(FROZEN_SDK)),
		});
		return Ok(downloads);
	}
	for (part, file) in absent {
		let name = format!("sdkshell_qemu_{platform}_{FIRMWARE_RELEASE}_{part}.bin");
		let (url, size, sha256) = github_asset("coredevices/PebbleOS", FIRMWARE_RELEASE, &name)?;
		downloads.push(Download {
			url,
			size,
			sha256,
			unpack: Unpack::File(dir.join(file)),
		});
	}
	Ok(downloads)
}

fn qemu_archive() -> Result<String> {
	let os = match std::env::consts::OS {
		"linux" => "linux",
		"macos" => "macos",
		os => bail!("Core doesn't build the emulator for {os}"),
	};
	let arch = match std::env::consts::ARCH {
		"x86_64" => "x86_64",
		"aarch64" => "arm64",
		arch => bail!("Core doesn't build the emulator for {arch}"),
	};
	Ok(format!("qemu-pebble-{os}-{arch}.tar.gz"))
}

/// List the downloads and ask before starting them.
fn confirm(platform: Platform, downloads: &[Download], yes: bool) -> Result<()> {
	let total: u64 = downloads.iter().map(|download| download.size).sum();
	eprintln!("{platform} emulator requires:");
	for download in downloads {
		eprintln!("  {:>9}  {}", megabytes(download.size), download.url);
	}
	if yes {
		return Ok(());
	}
	if !std::io::stdin().is_terminal() {
		bail!(
			"Not downloading {} without asking: pass --yes to allow it",
			megabytes(total)
		);
	}
	eprint!("Download {}? [y/N] ", megabytes(total));
	std::io::stderr().flush()?;
	let mut answer = String::new();
	std::io::stdin().read_line(&mut answer)?;
	if !matches!(answer.trim(), "y" | "Y" | "yes") {
		bail!("Not downloading the {platform} emulator");
	}
	Ok(())
}

fn fetch(download: &Download) -> Result<()> {
	let data = get_file(&download.url, download.size)?;
	verify(&data, &download.sha256, &download.url)?;
	match &download.unpack {
		Unpack::File(path) => write(path, &data),
		Unpack::Qemu(dir) => {
			let partial = dir.with_extension("partial");
			if partial.exists() {
				std::fs::remove_dir_all(&partial)?;
			}
			tar::Archive::new(flate2::read::GzDecoder::new(&data[..]))
				.unpack(&partial)
				.wrap_err("Can't unpack QEMU")?;
			std::fs::set_permissions(
				partial.join("bin/qemu-pebble"),
				std::fs::Permissions::from_mode(0o755),
			)?;
			std::fs::rename(&partial, dir)?;
			Ok(())
		}
		Unpack::FrozenSdk(dir) => frozen_firmware(&data, dir),
	}
}

/// The frozen platforms' firmware, all four at once, from the app SDK archive.
fn frozen_firmware(archive: &[u8], dir: &Path) -> Result<()> {
	let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(archive));
	for entry in tar.entries()? {
		let mut entry = entry?;
		let path = entry.path()?.into_owned();
		let Ok(rest) = path.strip_prefix("sdk-core/pebble") else {
			continue;
		};
		let parts: Vec<&str> = rest.iter().filter_map(|part| part.to_str()).collect();
		let [platform, "qemu", file] = parts[..] else {
			continue;
		};
		let Ok(platform) = platform.parse::<Platform>() else {
			continue;
		};
		if platform.config().frozen_at_revision.is_none() {
			continue;
		}
		let dir = dir.join(platform.as_str());
		let mut data = Vec::new();
		match file {
			"qemu_micro_flash.bin" => {
				entry.read_to_end(&mut data)?;
				write(&dir.join(MICRO_FLASH), &data)?;
			}
			"qemu_spi_flash.bin.bz2" => {
				bzip2::read::BzDecoder::new(entry).read_to_end(&mut data)?;
				write(&dir.join(SPI_FLASH), &data)?;
			}
			_ => {}
		}
	}
	Ok(())
}

/// The URL, size and SHA-256 of the asset `name` in `repo`'s release `tag`.
fn github_asset(repo: &str, tag: &str, name: &str) -> Result<(String, u64, String)> {
	let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
	let release: serde_json::Value = serde_json::from_str(
		&get(&url)?
			.body_mut()
			.read_to_string()
			.wrap_err_with(|| format!("Can't read {url}"))?,
	)?;
	let asset = release
		.get("assets")
		.and_then(serde_json::Value::as_array)
		.and_then(|assets| {
			assets
				.iter()
				.find(|asset| asset.get("name").and_then(serde_json::Value::as_str) == Some(name))
		})
		.ok_or_eyre(format!("{repo} {tag} has no {name}"))?;
	let field = |key: &str| asset.get(key).and_then(serde_json::Value::as_str);
	let url = field("browser_download_url").ok_or_eyre(format!("No download for {name}"))?;
	let size = asset
		.get("size")
		.and_then(serde_json::Value::as_u64)
		.ok_or_eyre(format!("No size for {name}"))?;
	let sha256 = field("digest")
		.and_then(|digest| digest.strip_prefix("sha256:"))
		.ok_or_eyre(format!("GitHub gives no SHA-256 for {name}"))?;
	Ok((url.to_owned(), size, sha256.to_owned()))
}

fn get(url: &str) -> Result<ureq::http::Response<ureq::Body>> {
	ureq::get(url)
		.header(
			"User-Agent",
			concat!("cargo-gneiss/", env!("CARGO_PKG_VERSION")),
		)
		.call()
		.map_err(|e| eyre!("Can't fetch {url}: {e}"))
}

/// Download `url`, about `size` bytes, drawing a progress bar if stderr is a terminal.
fn get_file(url: &str, size: u64) -> Result<Vec<u8>> {
	let name = url.rsplit('/').next().unwrap_or(url);
	let mut response = get(url)?;
	let mut reader = response
		.body_mut()
		.with_config()
		.limit(DOWNLOAD_LIMIT)
		.reader();
	let bar = std::io::stderr().is_terminal();
	if !bar {
		log::info!("Downloading {name}");
	}
	let mut data = Vec::with_capacity(usize::try_from(size).unwrap_or_default());
	let mut buf = vec![0; 64 << 10];
	let mut drawn: Option<Instant> = None;
	loop {
		let n = reader
			.read(&mut buf)
			.wrap_err_with(|| format!("Can't download {url}"))?;
		if n == 0 {
			break;
		}
		data.extend_from_slice(&buf[..n]);
		if bar && drawn.is_none_or(|at| at.elapsed() > Duration::from_millis(100)) {
			progress(name, data.len() as u64, size);
			drawn = Some(Instant::now());
		}
	}
	if bar {
		progress(name, data.len() as u64, size);
		eprintln!();
	}
	Ok(data)
}

fn progress(name: &str, done: u64, total: u64) {
	const WIDTH: u64 = 24;
	let filled = (done * WIDTH)
		.checked_div(total)
		.unwrap_or(WIDTH)
		.min(WIDTH);
	let percent = (done * 100).checked_div(total).unwrap_or(100).min(100);
	let bar: String = (0..WIDTH)
		.map(|i| if i < filled { '#' } else { '-' })
		.collect();
	eprint!(
		"\r  [{bar}] {:>9} / {:<9} {percent:>3}%  {name}",
		megabytes(done),
		megabytes(total)
	);
}

fn megabytes(bytes: u64) -> String {
	let tenths = (bytes + 50_000) / 100_000;
	format!("{}.{} MB", tenths / 10, tenths % 10)
}

fn verify(data: &[u8], sha256: &str, what: &str) -> Result<()> {
	let actual = Sha256::digest(data)
		.iter()
		.fold(String::new(), |mut hex, byte| {
			let _ = write!(hex, "{byte:02x}");
			hex
		});
	if !actual.eq_ignore_ascii_case(sha256) {
		bail!("{what} doesn't match its SHA-256: expected {sha256}, got {actual}");
	}
	Ok(())
}

/// Write `data` to `path` whole: a download cut short leaves no file.
fn write(path: &Path, data: &[u8]) -> Result<()> {
	let dir = path.parent().ok_or_eyre("Cache file has no directory")?;
	std::fs::create_dir_all(dir).wrap_err_with(|| format!("Can't create {}", dir.display()))?;
	let partial = path.with_extension("partial");
	std::fs::write(&partial, data)?;
	std::fs::rename(&partial, path).wrap_err_with(|| format!("Can't write {}", path.display()))
}

/// Downloads, which can all be fetched again
fn cache_dir() -> Result<PathBuf> {
	let base = match std::env::var_os("XDG_CACHE_HOME") {
		Some(dir) => PathBuf::from(dir),
		None => super::home()?.join(".cache"),
	};
	Ok(base.join("cargo-gneiss/emulator"))
}
