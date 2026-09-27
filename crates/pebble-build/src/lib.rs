//! Host-side build logic for Pebble apps: turn a compiled app ELF into its
//! per-platform `.bin` + `manifest.json`, and assemble the bundle's `appinfo`.
#![feature(iter_array_chunks)]
use std::{
	borrow::Borrow,
	cmp,
	collections::BTreeMap,
	convert::Infallible,
	env::var,
	ffi::OsStr,
	fmt::{Display, Write},
	path::{Path, PathBuf},
	str::FromStr,
};

use cargo_metadata::MetadataCommand;
pub use color_eyre;
use color_eyre::eyre::{Context, OptionExt, Result, bail};
use pebble_types::{Platform, pbpack::PbPack};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;

mod build_pbw;
mod pbl_font;
mod pbl_image;
mod pbl_svg;
mod pbl_vibe;
pub use build_pbw::{app_info, message_keys_json, package_platform};

use crate::{pbl_font::FontSettings, pbl_svg::SvgSettings};

/// Cargo sets `CARGO_FEATURE_<NAME>` for each enabled feature, so a platform's
/// selection env var follows from its name.
pub fn platform_features() -> impl Iterator<Item = (String, Platform)> {
	Platform::iter().map(|platform| {
		(
			format!("CARGO_FEATURE_{}", platform.as_str().to_uppercase()),
			platform,
		)
	})
}

/// Whether the app's `[features]` table declares `platform` as a build target
pub fn declares_platform(features: &BTreeMap<String, Vec<String>>, platform: Platform) -> bool {
	let name = platform.as_str();
	features.get(name).is_some_and(|forwards| {
		forwards.iter().any(|forward| {
			forward
				.strip_suffix(name)
				.and_then(|dep| dep.strip_suffix('/'))
				.is_some_and(|dep| dep == "gneiss" || dep == "pebble-sys")
		})
	})
}

/// Every platform `pkg` declares a feature for, in [`Platform`] order.
///
/// # Errors
/// If a feature named after a platform does not forward to it. Such an entry
/// would otherwise drop that watch from the bundle silently, or worse, forward
/// somewhere else and bundle another platform's binary under its name.
pub fn declared_platforms(pkg: &cargo_metadata::Package) -> Result<Vec<Platform>> {
	let (declared, misdeclared): (Vec<Platform>, Vec<Platform>) = Platform::iter()
		.filter(|platform| pkg.features.contains_key(platform.as_str()))
		.partition(|&platform| declares_platform(&pkg.features, platform));
	if !misdeclared.is_empty() {
		bail!(
			"{} declares {} without forwarding to the matching platform: each must contain \
			 `gneiss/<platform>` or `pebble-sys/<platform>`",
			pkg.name,
			misdeclared
				.iter()
				.map(|platform| format!("`{}`", platform.as_str()))
				.collect::<Vec<_>>()
				.join(", ")
		);
	}
	Ok(declared)
}

/// Pick the target platform.
/// `mutually_exclusive_features` only enforces at lib compile time — build.rs
/// runs first, so do the check here too.
/// # Panics
/// If multiple platforms are enabled by build time, or if the enabled platform is invalid
pub fn select_platform() -> Platform {
	let enabled: Vec<Platform> = platform_features()
		.filter_map(|(env, platform)| var(env).ok().map(|_| platform))
		.collect();
	match enabled.as_slice() {
		[p] => *p,
		[] => var("PBL_PLATFORM")
			.unwrap_or_else(|_| {
				panic!(
					"no platform selected: enable exactly one of the {} features, or set \
					 PBL_PLATFORM",
					Platform::iter()
						.map(|p| format!("`{}`", p.as_str()))
						.collect::<Vec<_>>()
						.join("/")
				)
			})
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
	/// The untagged source file. Siblings named `<stem>~<tag>…<ext>` override it
	/// on the platforms carrying those tags, so one declaration can carry a
	/// different file per watch, colour depth, shape or screen size. The tags
	/// are listed per platform in [`pebble_types::PlatformConfig`].
	pub file:             PathBuf,
	pub target_platforms: Vec<Platform>,
	#[serde(flatten)]
	pub res:              ResourceType,
}

/// A `<stem>~<tag>.<ext>` sibling of a declared resource file.
struct Variant {
	path:      PathBuf,
	is_named:  bool,
	tag_count: usize,
}

impl Resource {
	/// This resource's source file in `dir` for `platform`.
	///
	/// A sibling tagged with the platform's own name wins; failing that, the
	/// one carrying the most of the platform's other tags; failing that, the
	/// untagged `file`. Tags the platform does not carry rule a sibling out.
	///
	/// # Errors
	/// If `file` itself is tagged, if a sibling carries a tag no platform
	/// declares, or if two siblings are equally specific.
	pub fn resolve(&self, dir: &Path, platform: Platform) -> Result<PathBuf> {
		let untagged = dir.join(&self.file);
		let stem = self
			.file
			.file_stem()
			.ok_or_eyre(format!("Resource file {} has no name", self.file.display()))?
			.to_string_lossy()
			.into_owned();
		if stem.contains('~') {
			bail!(
				"Resource file {} is tagged; `file` names the untagged resource and tagged \
				 siblings are picked up from the directory",
				self.file.display()
			);
		}

		let Some(parent) = untagged.parent().map(Path::to_path_buf) else {
			return Ok(untagged);
		};
		let Ok(entries) = std::fs::read_dir(&parent) else {
			return Ok(untagged);
		};

		let mut best: Vec<Variant> = Vec::new();
		for entry in entries {
			let path = entry?.path();
			let Some(variant) = self.variant_at(&path, &stem, platform)? else {
				continue;
			};
			let better = best.first().is_none_or(|top| {
				(variant.is_named, variant.tag_count) > (top.is_named, top.tag_count)
			});
			if better {
				best.clear();
			}
			if better
				|| best.first().is_some_and(|top| {
					(variant.is_named, variant.tag_count) == (top.is_named, top.tag_count)
				}) {
				best.push(variant);
			}
		}

		match best.as_slice() {
			[] => Ok(untagged),
			[only] => Ok(only.path.clone()),
			ambiguous => bail!(
				"{} is ambiguous on {platform}: {} are equally specific",
				self.file.display(),
				ambiguous
					.iter()
					.map(|v| v.path.display().to_string())
					.collect::<Vec<_>>()
					.join(", ")
			),
		}
	}

	/// `path` as a variant of this resource for `platform`, or `None` if it is
	/// a different file or is tagged for other platforms.
	fn variant_at(&self, path: &Path, stem: &str, platform: Platform) -> Result<Option<Variant>> {
		if path.extension() != self.file.extension() {
			return Ok(None);
		}
		let Some(name) = path.file_stem().map(OsStr::to_string_lossy) else {
			return Ok(None);
		};
		let Some(tags) = name.strip_prefix(stem).and_then(|t| t.strip_prefix('~')) else {
			return Ok(None);
		};

		let mut is_named = false;
		let mut tag_count = 0;
		for tag in tags.split('~') {
			if !Platform::iter().any(|p| p.config().tags.contains(&tag)) {
				bail!(
					"{} carries the tag `{tag}`, which no platform declares",
					path.display()
				);
			}
			if !platform.config().tags.contains(&tag) {
				return Ok(None);
			}
			is_named |= tag == platform.as_str();
			tag_count += 1;
		}
		Ok(Some(Variant {
			path: path.to_path_buf(),
			is_named,
			tag_count,
		}))
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
	Svg(SvgSettings),
	Pdc,
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

/// Set the platform's `pbl_*` cfg flags, and declare the whole vocabulary so
/// `#[cfg]`-ing on a flag this platform lacks doesn't trip `unexpected_cfgs`.
/// A build script's cfgs reach only its own crate, so every crate that gates
/// on them calls this.
pub fn emit_platform_cfgs(platform: Platform) {
	for flag in pebble_types::ALL_CFGS {
		println!("cargo::rustc-check-cfg=cfg({flag})");
	}
	for flag in platform.config().cfgs {
		println!("cargo::rustc-cfg={flag}");
	}
}

/// To be called from app's `build.rs`, puts `resources.rs` and `message_keys.rs` in `OUT_DIR`.
pub fn gen_includes() -> Result<()> {
	for (env, _) in platform_features() {
		println!("cargo:rerun-if-env-changed={env}");
	}
	let in_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);

	let (_cargo_meta, pkg, mut pbl_metadata) = read_pbl_metadata()?;

	println!(
		"cargo:rerun-if-changed={}",
		pkg.manifest_path.join("Cargo.toml")
	);
	let platform = select_platform();

	emit_platform_cfgs(platform);

	gen_resource_declarations(&in_dir, &pbl_metadata, platform)?;
	gen_message_keys(pbl_metadata.message_keys.as_deref_mut())?;

	Ok(())
}

// TODO: Emit typed decls
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
	// Which file a resource resolves to depends on the directory's contents, not
	// only on the files this build read, so adding or dropping one must re-run it
	println!("cargo:rerun-if-changed={}", res_dir.display());
	for (name, resource) in &pbl_metadata.resources {
		if !resource.target_platforms.contains(&platform) {
			continue;
		}

		let path = resource.resolve(&res_dir, platform)?;
		println!("cargo:rerun-if-changed={}", path.display());
		let bytes = match resource.res.clone() {
			ResourceType::Raw => std::fs::read(&path).map_err(Into::into),
			ResourceType::Png => pbl_image::process_png(&path, platform),
			ResourceType::Bitmap => pbl_image::process_bitmap(&path, platform),
			ResourceType::Pbi => pbl_image::check_bitmap(&path),
			ResourceType::Font(settings) => {
				pbl_font::process_font(&path, name, platform, &settings)
			}
			ResourceType::Pbf => pbl_font::check_font(&path),
			ResourceType::Icon => pbl_image::process_icon(&path, platform),
			ResourceType::Vibe => pbl_vibe::process_vibe(&path),
			ResourceType::Svg(settings) => pbl_svg::process_svg(&path, &settings),
			ResourceType::Pdc => pbl_svg::check_pdc(&path),
		}
		.wrap_err_with(|| format!("{name} on {platform}, from {}", path.display()))?;
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
	if let Some(keys) = keys {
		// Same order + stride as `resolve_message_keys`, so the emitted indices
		// match the `appKeys`/`messageKeys` the companion reads.
		keys.sort();
		let mut index = 0u32;
		for key in keys.iter() {
			let name = key.name();
			let ctor = match key {
				MessageKey::Single(_) => format!("::gneiss::dict::SingleKey::new({index})"),
				MessageKey::Array { len, .. } => {
					format!("::gneiss::dict::ArrayKey::new({index}, {len})")
				}
			};
			let ty = match key {
				MessageKey::Single(_) => "::gneiss::dict::SingleKey",
				MessageKey::Array { .. } => "::gneiss::dict::ArrayKey",
			};
			let _ = writeln!(
				keys_rs,
				"#[allow(dead_code, non_upper_case_globals)]\n pub const {name}: {ty} = {ctor};"
			);
			index += key.len();
		}
	}
	let out = PathBuf::from(var("OUT_DIR")?);
	std::fs::write(out.join("message_keys.rs"), &keys_rs)?;
	Ok(())
}
