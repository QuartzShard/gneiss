//! Host-side build logic for Pebble apps: turn a compiled app ELF into its
//! per-platform `.bin` + `manifest.json`, and assemble the bundle's `appinfo`.
#![feature(iter_array_chunks)]
use std::{borrow::Borrow, collections::BTreeMap, env::var, path::PathBuf};

use cargo_metadata::MetadataCommand;
pub use color_eyre;
use color_eyre::eyre::{Context, OptionExt, Result, bail};
use pebble_types::{Platform, pbpack::PbPack};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod build_pbw;
mod pbl_font;
mod pbl_image;
mod pbl_vibe;
pub use build_pbw::{app_info, package_platform};

use crate::pbl_font::FontSettings;

pub const FEATURES: &[(&str, Platform)] = &[
	("CARGO_FEATURE_APLITE", Platform::Aplite),
	("CARGO_FEATURE_BASALT", Platform::Basalt),
	("CARGO_FEATURE_CHALK", Platform::Chalk),
	("CARGO_FEATURE_DIORITE", Platform::Diorite),
	("CARGO_FEATURE_EMERY", Platform::Emery),
	("CARGO_FEATURE_FLINT", Platform::Flint),
	("CARGO_FEATURE_GABBRO", Platform::Gabbro),
];

/// Pick the target platform. Prefer mutually-exclusive Cargo features; fall
/// back to the `PBL_PLATFORM` env var so `cargo xtask` flows (and anything
/// else not driven through `cargo build --features <plat>`) keep working.
/// `mutually_exclusive_features` only enforces at lib compile time — build.rs
/// runs first, so do the check here too.
pub fn select_platform() -> Platform {
	let enabled: Vec<Platform> = FEATURES
		.iter()
		.filter_map(|(env, p)| var(env).ok().map(|_| *p))
		.collect();
	match enabled.as_slice() {
		[p] => *p,
		[] => var("PBL_PLATFORM")
			.expect(
				"no platform selected: enable exactly one of the `aplite`/`basalt`/`chalk`/\
                 `diorite`/`emery`/`flint`/`gabbro` features, or set PBL_PLATFORM",
			)
			.parse()
			.expect("invalid PBL_PLATFORM"),
		many => panic!(
			"multiple platforms enabled ({:?}); choose exactly one",
			many.iter().map(|p| p.as_str()).collect::<Vec<_>>()
		),
	}
}
/// Contents of `[package.metadata.pebble]` — written by `cargo pebble new`,
/// read back at build time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PebbleMetadata {
	pub uuid:      Uuid,
	#[serde(default)]
	pub watchface: bool,
	pub resources: BTreeMap<String, Resource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
	pub file:             PathBuf,
	pub target_platforms: Vec<Platform>,
	#[serde(flatten)]
	pub res:              ResourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum ResourceType {
	Raw,
	Png,
	Bitmap,
	Pbi,
	Font(FontSettings),
	Pbf,
	/// A menu icon: processed as a bitmap, at most one per app, and its
	/// resource index is wired into the app header automatically
	Icon,
	Vibe,
}

/// Resource index of the app's `icon` resource in this platform's pack, if
/// declared. Bails on more than one
pub fn icon_index(meta: &PebbleMetadata, platform: Platform) -> Result<Option<u32>> {
	let mut icon = None;
	let mut pos = 0;
	for resource in meta.resources.values() {
		if !resource.target_platforms.contains(&platform) {
			continue;
		}
		pos += 1;
		if matches!(resource.res, ResourceType::Icon) && icon.replace(pos).is_some() {
			bail!("Multiple `icon` resources declared, at most one allowed");
		}
	}
	Ok(icon)
}
pub fn read_pbl_metadata() -> Result<
	(
		cargo_metadata::Metadata,
		cargo_metadata::Package,
		PebbleMetadata,
	),
	color_eyre::eyre::Error,
> {
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
	Ok((cargo_meta, pkg, pbl_metadata))
}

/// To be called from app's `build.rs`, puts `resources.rs` in OUT_DIR.
pub fn gen_resource_declarations() -> Result<()> {
	for (env, _) in FEATURES {
		println!("cargo:rerun-if-env-changed={env}");
	}
	let in_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);

	let (_cargo_meta, pkg, pbl_metadata) = read_pbl_metadata()?;

	println!(
		"cargo:rerun-if-changed={}",
		pkg.manifest_path.join("Cargo.toml")
	);
	let platform = select_platform();
	// Fail icon duplication at codegen time rather than at packaging
	icon_index(&pbl_metadata, platform)?;

	let mut resource_rs = String::from("// Generated consts for accessing declared resources\n");
	let mut pos = 0;
	let mut resources = Vec::with_capacity(pbl_metadata.resources.len());
	for (name, resource) in pbl_metadata.resources.iter() {
		if !resource.target_platforms.contains(&platform) {
			continue;
		};

		let path = in_dir.join("resources").join(&resource.file);
		println!("cargo:rerun-if-changed={}", path.display());
		let bytes = match resource.res.clone() {
			ResourceType::Raw => std::fs::read(&path)
				.wrap_err_with(|| format!("Resource Missing! {}", path.display()))?,
			ResourceType::Png => pbl_image::process_png(&path, platform)?,
			ResourceType::Bitmap => pbl_image::process_bitmap(&path, platform)?,
			ResourceType::Pbi => pbl_image::check_bitmap(&path)?,
			ResourceType::Font(settings) => {
				pbl_font::process_font(&path, name, platform, settings)?
			}
			ResourceType::Pbf => pbl_font::check_font(&path)?,
			ResourceType::Icon => pbl_image::process_icon(&path, platform)?,
			ResourceType::Vibe => pbl_vibe::process_vibe(&path)?,
		};
		pos += 1;
		// Not all resources are referenced from app code; icons in particular
		// are consumed by the packaging step
		resource_rs.push_str(&format!(
			"#[allow(dead_code)]\npub const {name}: u32 = {pos};\n"
		));
		resources.push(bytes);
	}
	let resources: Vec<&[u8]> = resources.iter().map(Borrow::borrow).collect();
	let pbpack = PbPack::new(&resources);
	let out = PathBuf::from(var("OUT_DIR")?);
	std::fs::write(out.join("resources.rs"), &resource_rs)?;
	std::fs::write(
		out.join(format!("{}_app_resources.pbpack", platform)),
		pbpack.as_bytes(),
	)?;
	Ok(())
}
