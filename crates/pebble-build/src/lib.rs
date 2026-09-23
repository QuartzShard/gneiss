//! Host-side build logic for Pebble apps: turn a compiled app ELF into its
//! per-platform `.bin` + `manifest.json`, and assemble the bundle's `appinfo`.
#![feature(iter_array_chunks)]
use std::{
	borrow::Borrow,
	cmp,
	collections::BTreeMap,
	convert::Infallible,
	env::var,
	ffi::OsString,
	fmt::{Display, Write},
	path::{Path, PathBuf},
	str::FromStr,
};

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
pub use build_pbw::{app_info, message_keys_json, package_platform};

use crate::{build_pbw::resolve_message_keys, pbl_font::FontSettings};

pub const FEATURES: &[(&str, Platform)] = &[
	("CARGO_FEATURE_APLITE", Platform::Aplite),
	("CARGO_FEATURE_BASALT", Platform::Basalt),
	("CARGO_FEATURE_CHALK", Platform::Chalk),
	("CARGO_FEATURE_DIORITE", Platform::Diorite),
	("CARGO_FEATURE_EMERY", Platform::Emery),
	("CARGO_FEATURE_FLINT", Platform::Flint),
	("CARGO_FEATURE_GABBRO", Platform::Gabbro),
];

/// Pick the target platform.
/// `mutually_exclusive_features` only enforces at lib compile time — build.rs
/// runs first, so do the check here too.
/// # Panics
/// If multiple platforms are enabled by build time, or if the enabled platform is invalid
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
	pub uuid:         Uuid,
	#[serde(default)]
	pub watchface:    bool,
	pub resources:    BTreeMap<String, Resource>,
	pub message_keys: Option<Vec<MessageKey>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
	/// The shared source file. A sibling `<stem>~<platform>.<ext>` wins over it
	/// on the platform it names, so one declaration can carry a different file
	/// per watch
	pub file:             PathBuf,
	pub target_platforms: Vec<Platform>,
	#[serde(flatten)]
	pub res:              ResourceType,
}

impl Resource {
	/// This resource's source file in `dir` for `platform`: its
	/// `<stem>~<platform>.<ext>` variant if one exists, `file` otherwise.
	pub fn resolve(&self, dir: &Path, platform: Platform) -> PathBuf {
		let variant = dir.join(self.variant_of(platform));
		if variant.is_file() {
			variant
		} else {
			dir.join(&self.file)
		}
	}

	fn variant_of(&self, platform: Platform) -> PathBuf {
		let mut name = self
			.file
			.file_stem()
			.map_or_else(OsString::new, ToOwned::to_owned);
		name.push("~");
		name.push(platform.as_str());
		if let Some(ext) = self.file.extension() {
			name.push(".");
			name.push(ext);
		}
		self.file.with_file_name(name)
	}
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

#[derive(Debug, Clone)]
pub enum MessageKey {
	Single(String),
	Array { name: String, len: usize },
}

impl MessageKey {
	pub fn name(&self) -> &str {
		match self {
			MessageKey::Single(name) | MessageKey::Array { name, .. } => name,
		}
	}

	pub fn len(&self) -> u32 {
		match self {
			MessageKey::Single(_) => 1,
			MessageKey::Array { len, .. } => *len as u32,
		}
	}
}

impl Display for MessageKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MessageKey::Single(name) => write!(f, "{name}"),
			MessageKey::Array { name, len } => write!(f, "{name}[{len}]"),
		}
	}
}

impl FromStr for MessageKey {
	type Err = Infallible;

	fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
		let (name, len) = s
			.split_once('[')
			.and_then(|(name, rest)| rest.split_once(']').map(|(len, _)| (name, len)))
			.map_or_else(
				|| (s.to_string(), None),
				|(name, len)| (name.to_string(), len.parse().ok()),
			);
		Ok(if let Some(len) = len {
			Self::Array { name, len }
		} else {
			Self::Single(name)
		})
	}
}

impl serde::Serialize for MessageKey {
	fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.collect_str(self)
	}
}

impl<'de> serde::Deserialize<'de> for MessageKey {
	fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		String::deserialize(deserializer)?
			.parse()
			.map_err(serde::de::Error::custom)
	}
}

impl Ord for MessageKey {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		match (self, other) {
			(MessageKey::Single(name), MessageKey::Single(other)) => name.cmp(other),
			(MessageKey::Single(_), MessageKey::Array { .. }) => cmp::Ordering::Greater,
			(MessageKey::Array { .. }, MessageKey::Single(_)) => cmp::Ordering::Less,
			(MessageKey::Array { name, .. }, MessageKey::Array { name: other, .. }) => {
				name.cmp(other)
			}
		}
	}
}

impl PartialOrd for MessageKey {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl PartialEq for MessageKey {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Single(l0), Self::Single(r0)) => l0 == r0,
			(Self::Array { name: l_name, .. }, Self::Array { name: r_name, .. }) => {
				l_name == r_name
			}
			_ => false,
		}
	}
}
impl Eq for MessageKey {}
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

/// To be called from app's `build.rs`, puts `resources.rs` and `message_keys.rs` in `OUT_DIR`.
pub fn gen_includes() -> Result<()> {
	for (env, _) in FEATURES {
		println!("cargo:rerun-if-env-changed={env}");
	}
	let in_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);

	let (_cargo_meta, pkg, mut pbl_metadata) = read_pbl_metadata()?;

	println!(
		"cargo:rerun-if-changed={}",
		pkg.manifest_path.join("Cargo.toml")
	);
	let platform = select_platform();

	gen_resource_declarations(&in_dir, &pbl_metadata, platform)?;
	gen_message_keys(pbl_metadata.message_keys.as_deref_mut())?;

	Ok(())
}

pub fn gen_resource_declarations(
	in_dir: &Path,
	pbl_metadata: &PebbleMetadata,
	platform: Platform,
) -> Result<()> {
	// Fail icon duplication at codegen time rather than at packaging
	icon_index(pbl_metadata, platform)?;

	let mut resource_rs = String::from("// Generated consts for accessing declared resources\n");
	let mut pos = 0;
	let mut resources = Vec::with_capacity(pbl_metadata.resources.len());
	let res_dir = in_dir.join("resources");
	// Whether a `~platform` variant wins depends on the directory's contents, not
	// just on the files this build read, so adding or dropping one has to re-run us
	println!("cargo:rerun-if-changed={}", res_dir.display());
	for (name, resource) in &pbl_metadata.resources {
		if !resource.target_platforms.contains(&platform) {
			continue;
		}

		let path = resource.resolve(&res_dir, platform);
		println!("cargo:rerun-if-changed={}", path.display());
		let bytes = match resource.res.clone() {
			ResourceType::Raw => std::fs::read(&path)
				.wrap_err_with(|| format!("Resource Missing! {}", path.display()))?,
			ResourceType::Png => pbl_image::process_png(&path, platform)?,
			ResourceType::Bitmap => pbl_image::process_bitmap(&path, platform)?,
			ResourceType::Pbi => pbl_image::check_bitmap(&path)?,
			ResourceType::Font(settings) => {
				pbl_font::process_font(&path, name, platform, &settings)?
			}
			ResourceType::Pbf => pbl_font::check_font(&path)?,
			ResourceType::Icon => pbl_image::process_icon(&path, platform)?,
			ResourceType::Vibe => pbl_vibe::process_vibe(&path)?,
		};
		pos += 1;
		// Not all resources are referenced from app code; icons in particular
		// are consumed by the packaging step
		let _ = write!(
			resource_rs,
			"#[allow(dead_code)]\npub const {name}: u32 = {pos};\n"
		);
		resources.push(bytes);
	}
	let resources: Vec<&[u8]> = resources.iter().map(Borrow::borrow).collect();
	let pbpack = PbPack::new(&resources);
	let out = PathBuf::from(var("OUT_DIR")?);
	std::fs::write(out.join("resources.rs"), &resource_rs)?;
	std::fs::write(
		out.join(format!("{platform}_app_resources.pbpack")),
		pbpack.as_bytes(),
	)?;
	Ok(())
}

pub fn gen_message_keys(keys: Option<&mut [MessageKey]>) -> Result<()> {
	let mut keys_rs = String::from("// Generated consts for accessing declared message_keys\n");
	for (key, index) in resolve_message_keys(keys) {
		let _ = writeln!(
			keys_rs,
			"#[allow(dead_code, non_upper_case_globals)]\n pub const {key}: u32 = {index};"
		);
	}
	let out = PathBuf::from(var("OUT_DIR")?);
	std::fs::write(out.join("message_keys.rs"), &keys_rs)?;
	Ok(())
}
