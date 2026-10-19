//! Where the emulator and its firmware images come from: for now, the SDK pebble-tool last
//! installed.
use std::path::PathBuf;

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail},
};
use gneiss_types::Platform;

pub struct Assets {
	pub qemu:          PathBuf,
	/// The firmware QEMU boots
	pub micro_flash:   PathBuf,
	/// A fresh SPI flash image, bzip2-compressed
	pub spi_flash_bz2: PathBuf,
	/// The firmware's version, which a flash image belongs to
	pub version:       String,
}

pub fn locate(platform: Platform) -> Result<Assets> {
	let current = super::home()?.join(".pebble-sdk/SDKs/current");
	let sdk = current.canonicalize().wrap_err_with(|| {
		format!(
			"No SDK at {}: install one with pebble-tool, for now",
			current.display()
		)
	})?;
	let version = sdk
		.file_name()
		.and_then(|name| name.to_str())
		.ok_or_eyre(format!("Can't tell the version of {}", sdk.display()))?
		.to_owned();
	let images = sdk
		.join("sdk-core/pebble")
		.join(platform.as_str())
		.join("qemu");
	let assets = Assets {
		qemu: sdk.join("toolchain/bin/qemu-pebble"),
		micro_flash: images.join("qemu_micro_flash.bin"),
		spi_flash_bz2: images.join("qemu_spi_flash.bin.bz2"),
		version,
	};
	for path in [&assets.qemu, &assets.micro_flash, &assets.spi_flash_bz2] {
		if !path.exists() {
			bail!("SDK {} has no {}", assets.version, path.display());
		}
	}
	Ok(assets)
}
