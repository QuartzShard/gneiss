//! Upstream's platform table -> `pebble-types`'s `config.rs`.
//!
//! `snapshot-vendor` dumps `tools/pebble_sdk_platform.py` to
//! `vendor/platforms.json`. Everything `PlatformConfig` carries comes from
//! there, except the process-info platform bit, which is not in upstream's
//! table: that comes from the `pebble_process_info.h` bindings generated in
//! the same run.
use std::{collections::BTreeSet, path::Path};

use color_eyre::eyre::{Context, OptionExt, Result, bail};
use pebble_types::Platform;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use strum::IntoEnumIterator;

/// One platform as upstream writes it. The keys it also carries and we do not
/// read: `BUILD_DIR`, `BUNDLE_BIN_DIR`, `ADDITIONAL_TEXT_LINES_FOR_PEBBLE_H`,
/// `HAS_MODDABLE_XS`, `MAX_RESOURCES_SIZE_APPSTORE_2_X`.
#[derive(Debug, Deserialize)]
pub struct UpstreamPlatform {
	#[serde(rename = "NAME")]
	pub name: String,
	#[serde(rename = "DEFINES")]
	pub defines: Vec<String>,
	#[serde(rename = "TAGS")]
	pub tags: Vec<String>,
	#[serde(rename = "FROZEN_AT_REVISION")]
	pub frozen_at_revision: Option<u32>,
	#[serde(rename = "MAX_APP_BINARY_SIZE")]
	pub max_app_binary_size: u32,
	#[serde(rename = "MAX_APP_MEMORY_SIZE")]
	pub max_app_memory_size: u32,
	#[serde(rename = "MAX_WORKER_MEMORY_SIZE")]
	pub max_worker_memory_size: u32,
	#[serde(rename = "MAX_RESOURCES_SIZE")]
	pub max_resources_size: u32,
	#[serde(rename = "MAX_RESOURCES_SIZE_APPSTORE")]
	pub max_resources_size_appstore: u32,
	#[serde(rename = "MAX_FONT_GLYPH_SIZE")]
	pub max_font_glyph_size: u16,
}

/// The vendored table, classified and ordered by platform name. Loading
/// classifies, so an unknown define fails before anything is written.
pub struct PlatformTable(Vec<(Platform, UpstreamPlatform, Classified)>);

/// `PBL_*` define -> `Capabilities` field, the one mapping kept by hand here.
/// Upstream's tag names do not track the defines:
/// `PBL_MICROPHONE` tags as `mic`, `PBL_SMARTSTRAP_POWER` as `strappower`, and
/// `PBL_RGB_BACKLIGHT` carries no tag at all.
const CAPABILITIES: &[(&str, &str)] = &[
	("PBL_MICROPHONE", "microphone"),
	("PBL_SMARTSTRAP", "smartstrap"),
	("PBL_SMARTSTRAP_POWER", "smartstrap_power"),
	("PBL_HEALTH", "health"),
	("PBL_COMPASS", "compass"),
	("PBL_TOUCH", "touch"),
	("PBL_RGB_BACKLIGHT", "rgb_backlight"),
	("PBL_SPEAKER", "speaker"),
];

/// Doc comments for the emitted `Capabilities` fields. Fields without an
/// entry get none.
const CAPABILITY_DOCS: &[(&str, &str)] = &[
	(
		"color",
		"`false` means BW (`PBL_BW`); `true` means color (`PBL_COLOR`).",
	),
	(
		"smartstrap_power",
		"Implies `smartstrap`. `PBL_SMARTSTRAP_POWER`.",
	),
];

/// Defines consumed by a dedicated field rather than a `Capabilities` bool:
/// the polar pairs become `color` and `display.shape`, `PBL_SDK_FROZEN` pairs
/// with `FROZEN_AT_REVISION`. `PBL_PLATFORM_<NAME>` is checked separately.
const STRUCTURAL: &[&str] = &[
	"PBL_COLOR",
	"PBL_BW",
	"PBL_RECT",
	"PBL_ROUND",
	"PBL_SDK_FROZEN",
];

impl PlatformTable {
	pub fn load(vendor: &Path) -> Result<Self> {
		let path = vendor.join("platforms.json");
		let raw = std::fs::read_to_string(&path).wrap_err_with(|| {
			format!(
				"{} missing — run `cargo xtask snapshot-vendor` to refresh the snapshot",
				path.display()
			)
		})?;
		let mut by_name: std::collections::BTreeMap<String, UpstreamPlatform> =
			serde_json::from_str(&raw).wrap_err_with(|| format!("Malformed {}", path.display()))?;

		let mut table = Vec::new();
		for platform in Platform::iter() {
			let upstream = by_name
				.remove(platform.as_str())
				.ok_or_eyre(format!("{platform} missing from {}", path.display()))?;
			if upstream.name != platform.as_str() {
				bail!(
					"{platform}: upstream NAME is {:?}, expected {:?}",
					upstream.name,
					platform.as_str()
				);
			}
			let classified = classify(platform, &upstream)?;
			table.push((platform, upstream, classified));
		}
		table.sort_by_key(|(platform, ..)| platform.as_str());
		Ok(Self(table))
	}

	pub fn get(&self, platform: Platform) -> Result<&UpstreamPlatform> {
		self.0
			.iter()
			.find_map(|(p, upstream, _)| (*p == platform).then_some(upstream))
			.ok_or_eyre(format!(
				"{platform} missing from the vendored platform table"
			))
	}

	/// `-D` arguments for a platform, straight from the vendored table.
	pub fn clang_args(&self, platform: Platform) -> Result<Vec<String>> {
		Ok(self
			.get(platform)?
			.defines
			.iter()
			.map(|define| format!("-D{define}"))
			.collect())
	}
}

/// A platform's defines, classified.
#[derive(Debug)]
struct Classified {
	color:        bool,
	round:        bool,
	width:        u16,
	height:       u16,
	/// Capability field -> set, in `CAPABILITIES` order
	capabilities: Vec<(&'static str, bool)>,
	/// `defines` minus the `NAME=value` ones, lowercased
	cfgs:         Vec<String>,
}

fn classify(platform: Platform, upstream: &UpstreamPlatform) -> Result<Classified> {
	let defines: BTreeSet<&str> = upstream.defines.iter().map(String::as_str).collect();
	let platform_define = format!("PBL_PLATFORM_{}", platform.as_str().to_uppercase());
	if !defines.contains(platform_define.as_str()) {
		bail!("{platform}: no {platform_define} in defines");
	}

	let mut width = None;
	let mut height = None;
	for define in &defines {
		if *define == platform_define
			|| STRUCTURAL.contains(define)
			|| CAPABILITIES.iter().any(|(known, _)| known == define)
		{
			continue;
		}
		match define.split_once('=') {
			Some(("PBL_DISPLAY_WIDTH", value)) => width = Some(value.parse()?),
			Some(("PBL_DISPLAY_HEIGHT", value)) => height = Some(value.parse()?),
			_ => bail!(
				"{platform}: upstream define `{define}` has no rule in \
				 xtask/src/platform_table.rs.\nIf it names a capability, add it to \
				 `CAPABILITIES`; if it describes the platform, add it to `STRUCTURAL` and \
				 consume it in `classify`."
			),
		}
	}

	let exactly_one = |a: &str, b: &str| -> Result<bool> {
		match (defines.contains(a), defines.contains(b)) {
			(true, false) => Ok(true),
			(false, true) => Ok(false),
			(true, true) => bail!("{platform}: both {a} and {b} defined"),
			(false, false) => bail!("{platform}: neither {a} nor {b} defined"),
		}
	};

	let frozen = defines.contains("PBL_SDK_FROZEN");
	if frozen != upstream.frozen_at_revision.is_some() {
		bail!(
			"{platform}: PBL_SDK_FROZEN is {frozen} but FROZEN_AT_REVISION is {:?}",
			upstream.frozen_at_revision
		);
	}

	let capabilities: Vec<(&'static str, bool)> = CAPABILITIES
		.iter()
		.map(|(define, field)| (*field, defines.contains(define)))
		.collect();
	let set = |field: &str| {
		capabilities
			.iter()
			.any(|(name, set)| *name == field && *set)
	};
	// Upstream sets both defines on a powered-smartstrap platform
	if set("smartstrap_power") && !set("smartstrap") {
		bail!("{platform}: PBL_SMARTSTRAP_POWER without PBL_SMARTSTRAP");
	}

	// Upstream's DEFINES order varies per platform; sorted for a stable list
	let cfgs = defines
		.iter()
		.filter(|define| !define.contains('='))
		.map(|define| define.to_lowercase())
		.collect();

	Ok(Classified {
		color: exactly_one("PBL_COLOR", "PBL_BW")?,
		round: !exactly_one("PBL_RECT", "PBL_ROUND")?,
		width: width.ok_or_eyre(format!("{platform}: no PBL_DISPLAY_WIDTH"))?,
		height: height.ok_or_eyre(format!("{platform}: no PBL_DISPLAY_HEIGHT"))?,
		capabilities,
		cfgs,
	})
}

/// `PROCESS_INFO_CURRENT_SDK_VERSION_*` out of each platform's own vendored
/// `pebble_process_info.h`. The headers are identical bar these two: a frozen
/// platform's firmware knows an older applib, and rejects an app whose header
/// claims a newer one.
pub fn sdk_versions(vendor: &Path) -> Result<Vec<(Platform, (u8, u8))>> {
	fn define(header: &str, name: &str) -> Option<u8> {
		let value = header
			.lines()
			.find_map(|line| line.trim().strip_prefix(&format!("#define {name} ")))?
			.trim();
		let digits = value.strip_prefix("0x")?;
		u8::from_str_radix(digits, 16).ok()
	}

	let mut versions = Vec::new();
	for platform in Platform::iter() {
		let path = vendor.join(platform.as_str()).join("pebble_process_info.h");
		let header = std::fs::read_to_string(&path)
			.wrap_err_with(|| format!("Cannot read {}", path.display()))?;
		let major = define(&header, "PROCESS_INFO_CURRENT_SDK_VERSION_MAJOR");
		let minor = define(&header, "PROCESS_INFO_CURRENT_SDK_VERSION_MINOR");
		let (Some(major), Some(minor)) = (major, minor) else {
			bail!("No PROCESS_INFO_CURRENT_SDK_VERSION in {}", path.display());
		};
		versions.push((platform, (major, minor)));
	}
	Ok(versions)
}

/// `PROCESS_INFO_PLATFORM_<NAME>` values out of the generated
/// `pebble_process_info.rs`. The header packs them as `N << 6` inside
/// `PebbleProcessInfoFlags`; `PlatformConfig` keeps the unshifted `N`.
pub fn process_info_bits(bindings: &str) -> Result<Vec<(Platform, u8)>> {
	const PREFIX: &str = "pub const PebbleProcessInfoFlags_PROCESS_INFO_PLATFORM_";

	let mut bits = Vec::new();
	for platform in Platform::iter() {
		let needle = format!("{PREFIX}{}:", platform.as_str().to_uppercase());
		let value: u32 = bindings
			.lines()
			.find_map(|line| line.trim().strip_prefix(&needle))
			.and_then(|rest| rest.rsplit_once('=').map(|(_, value)| value))
			.ok_or_eyre(format!(
				"{platform}: no PROCESS_INFO_PLATFORM constant in the generated \
				 pebble_process_info bindings"
			))?
			.trim()
			.trim_end_matches(';')
			.parse()?;
		if value % 64 != 0 || !(64..=448).contains(&value) {
			bail!("{platform}: PROCESS_INFO_PLATFORM value {value} is not N << 6 for N in 1..=7");
		}
		bits.push((platform, (value >> 6) as u8));
	}
	Ok(bits)
}
/// Render the whole of `pebble-types`'s `config.rs`.
pub fn render(
	table: &PlatformTable,
	bits: &[(Platform, u8)],
	sdk: &[(Platform, (u8, u8))],
) -> Result<String> {
	let capabilities = capabilities_struct();
	let consts = table
		.0
		.iter()
		.map(|(platform, upstream, classified)| {
			platform_const(*platform, upstream, classified, bits, sdk)
		})
		.collect::<Result<Vec<_>>>()?;
	let all_cfgs: BTreeSet<&str> = table
		.0
		.iter()
		.flat_map(|(.., classified)| classified.cfgs.iter().map(String::as_str))
		.collect();
	let all_cfgs = all_cfgs.iter();
	let arms = table.0.iter().map(|(platform, ..)| {
		let variant = format_ident!("{platform:?}");
		let konst = platform_ident(*platform);
		quote!(Platform::#variant => &#konst,)
	});

	let tokens = quote! {
		use crate::{Platform, pebble_process_info::Version};

		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub enum DisplayShape {
			Rect,
			Round,
		}

		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub struct Display {
			pub width: u16,
			pub height: u16,
			pub shape: DisplayShape,
		}

		#capabilities

		#[derive(Debug, Clone, Copy)]
		pub struct PlatformConfig {
			#[doc = " Upper-case canonical name, e.g. `\"EMERY\"`."]
			pub name: &'static str,
			#[doc = " The applib version this platform's firmware knows, from its own"]
			#[doc = " `pebble_process_info.h`. An app header claiming a newer one is"]
			#[doc = " rejected at install, so the frozen watches sit below the current"]
			#[doc = " SDK's."]
			pub sdk_version: Version,
			#[doc = " Raw `N` from `PROCESS_INFO_PLATFORM_<NAME> = N << 6`. The OS shifts it"]
			#[doc = " into position; we store the unshifted value to make the relationship"]
			#[doc = " explicit at use-time."]
			pub process_info_platform_bit: u8,
			pub max_app_binary_size: u32,
			#[doc = " Becomes `LENGTH` of the `APP` memory region in the app linker script."]
			pub max_app_memory_size: u32,
			pub max_worker_memory_size: u32,
			pub max_resources_size: u32,
			pub max_resources_size_appstore: u32,
			pub max_font_glyph_size: u16,
			pub display: Display,
			#[doc = " `Some(rev)` only for legacy platforms whose ABI is frozen."]
			#[doc = " Mirrors the presence of `PBL_SDK_FROZEN` in `defines`."]
			pub frozen_at_revision: Option<u32>,
			#[doc = " Bare define names, ready for `-D` prefixing. Includes the platform"]
			#[doc = " name (`PBL_PLATFORM_<NAME>`), color/shape (`PBL_COLOR`|`PBL_BW`,"]
			#[doc = " `PBL_RECT`|`PBL_ROUND`), capability flags, `PBL_DISPLAY_WIDTH=<n>` /"]
			#[doc = " `PBL_DISPLAY_HEIGHT=<n>`, and `PBL_SDK_FROZEN` where applicable."]
			#[doc = " `PBL_SDK_3` is universal and is added by the consumer (xtask), not here."]
			pub defines: &'static [&'static str],
			#[doc = " Upstream's resource tags for this platform: the platform name, colour"]
			#[doc = " depth, shape, capabilities and dimensions. A resource file suffixed"]
			#[doc = " `~<tag>` applies to the platforms carrying that tag."]
			pub tags: &'static [&'static str],
			#[doc = " `defines` lowercased, minus the `NAME=value` ones: the cfg flags a"]
			#[doc = " build script sets for this platform (`cargo::rustc-cfg=pbl_touch`)."]
			#[doc = " [`ALL_CFGS`] is the matching `check-cfg` vocabulary."]
			pub cfgs: &'static [&'static str],
			pub capabilities: Capabilities,
		}

		#(#consts)*

		#[doc = " Every cfg flag any platform sets, for a build script's"]
		#[doc = " `cargo::rustc-check-cfg` lines. A flag the current platform does not"]
		#[doc = " set still needs declaring, or `#[cfg]` on it warns under"]
		#[doc = " `unexpected_cfgs`."]
		pub const ALL_CFGS: &[&str] = &[#(#all_cfgs),*];

		impl Platform {
			pub const fn config(self) -> &'static PlatformConfig {
				match self {
					#(#arms)*
				}
			}
		}

		#[cfg(test)]
		#[path = "config_tests.rs"]
		mod tests;
	};

	// syn takes inner attributes only at the head of a file, not from a token
	// stream, so the module docs go on afterwards
	let mut file: syn::File = syn::parse2(tokens)?;
	file.attrs = vec![
		syn::parse_quote!(#![doc = " Per-platform build/runtime configuration."]),
		syn::parse_quote!(#![doc = ""]),
		syn::parse_quote!(
			#![doc = " Generated by `cargo xtask regen-sys` from `pebble-sys/vendor/platforms.json`"]
		),
		syn::parse_quote!(
			#![doc = " (upstream `tools/pebble_sdk_platform.py`; see `vendor/UPSTREAM` for the"]
		),
		syn::parse_quote!(#![doc = " snapshot's commit) and the `pebble_process_info.h` bindings."]),
		syn::parse_quote!(
			#![doc = " Do not hand-edit: see `xtask/src/platform_table.rs`"]
		),
	];
	Ok(prettyplease::unparse(&file))
}

/// `Capabilities`, one `bool` per row of [`CAPABILITIES`] plus `color`.
fn capabilities_struct() -> TokenStream {
	let fields = capability_fields().map(|field| {
		let ident = format_ident!("{field}");
		let docs = CAPABILITY_DOCS
			.iter()
			.filter(move |(name, _)| *name == field)
			.map(|(_, doc)| format!(" {doc}"));
		quote! {
			#(#[doc = #docs])*
			pub #ident: bool,
		}
	});
	quote! {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub struct Capabilities {
			#(#fields)*
		}
	}
}

/// Field names of the emitted `Capabilities`, in declaration order.
fn capability_fields() -> impl Iterator<Item = &'static str> {
	std::iter::once("color").chain(CAPABILITIES.iter().map(|(_, field)| *field))
}

fn platform_ident(platform: Platform) -> proc_macro2::Ident {
	format_ident!("PLATFORM_{}", platform.as_str().to_uppercase())
}

/// `quote!` renders integers in decimal; upstream writes the sizes in hex, so
/// they are emitted as literal tokens.
fn hex(value: u32) -> syn::LitInt {
	syn::LitInt::new(&format!("{value:#X}"), Span::call_site())
}

fn int(value: impl Into<u64>) -> proc_macro2::Literal {
	proc_macro2::Literal::u64_unsuffixed(value.into())
}

fn platform_const(
	platform: Platform,
	upstream: &UpstreamPlatform,
	classified: &Classified,
	bits: &[(Platform, u8)],
	sdk: &[(Platform, (u8, u8))],
) -> Result<TokenStream> {
	let konst = platform_ident(platform);
	let name = platform.as_str().to_uppercase();
	let bit = int(*bits
		.iter()
		.find_map(|(p, bit)| (*p == platform).then_some(bit))
		.ok_or_eyre(format!("{platform}: no process-info bit"))?);
	let (sdk_major, sdk_minor) = sdk
		.iter()
		.find_map(|(p, version)| (*p == platform).then_some(*version))
		.ok_or_eyre(format!("{platform}: no SDK version"))?;
	let (sdk_major, sdk_minor) = (int(sdk_major), int(sdk_minor));

	let (binary, memory, worker, resources, appstore) = (
		hex(upstream.max_app_binary_size),
		hex(upstream.max_app_memory_size),
		hex(upstream.max_worker_memory_size),
		hex(upstream.max_resources_size),
		hex(upstream.max_resources_size_appstore),
	);
	let glyph = int(upstream.max_font_glyph_size);
	let (width, height) = (int(classified.width), int(classified.height));
	let shape = format_ident!("{}", if classified.round { "Round" } else { "Rect" });
	let frozen = match upstream.frozen_at_revision {
		Some(revision) => {
			let revision = int(revision);
			quote!(Some(#revision))
		}
		None => quote!(None),
	};
	let (defines, tags, cfgs) = (&upstream.defines, &upstream.tags, &classified.cfgs);
	let color = classified.color;
	let caps = classified.capabilities.iter().map(|(field, set)| {
		let ident = format_ident!("{field}");
		quote!(#ident: #set,)
	});

	Ok(quote! {
		pub const #konst: PlatformConfig = PlatformConfig {
			name: #name,
			process_info_platform_bit: #bit,
			sdk_version: Version { major: #sdk_major, minor: #sdk_minor },
			max_app_binary_size: #binary,
			max_app_memory_size: #memory,
			max_worker_memory_size: #worker,
			max_resources_size: #resources,
			max_resources_size_appstore: #appstore,
			max_font_glyph_size: #glyph,
			display: Display {
				width: #width,
				height: #height,
				shape: DisplayShape::#shape,
			},
			frozen_at_revision: #frozen,
			defines: &[#(#defines),*],
			tags: &[#(#tags),*],
			cfgs: &[#(#cfgs),*],
			capabilities: Capabilities {
				color: #color,
				#(#caps)*
			},
		};
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	fn upstream(defines: &[&str]) -> UpstreamPlatform {
		UpstreamPlatform {
			name: "emery".into(),
			defines: defines.iter().map(|d| (*d).to_string()).collect(),
			tags: vec!["emery".into()],
			frozen_at_revision: None,
			max_app_binary_size: 0x2_0000,
			max_app_memory_size: 0x2_0000,
			max_worker_memory_size: 0x2800,
			max_resources_size: 0x10_0000,
			max_resources_size_appstore: 0x4_0000,
			max_font_glyph_size: 512,
		}
	}

	const MINIMAL: &[&str] = &[
		"PBL_PLATFORM_EMERY",
		"PBL_COLOR",
		"PBL_RECT",
		"PBL_DISPLAY_WIDTH=200",
		"PBL_DISPLAY_HEIGHT=228",
	];

	#[test]
	fn an_unknown_define_is_a_hard_error() {
		let mut defines = MINIMAL.to_vec();
		defines.push("PBL_ALTIMETER");
		let err = classify(Platform::Emery, &upstream(&defines))
			.expect_err("an unmapped capability must not be silently dropped");
		let msg = err.to_string();
		assert!(msg.contains("PBL_ALTIMETER"), "unhelpful error: {msg}");
		assert!(msg.contains("CAPABILITIES"), "unhelpful error: {msg}");
	}

	#[test]
	fn a_known_define_set_classifies() {
		let cls = classify(Platform::Emery, &upstream(MINIMAL)).expect("minimal defines");
		assert!(cls.color);
		assert!(!cls.round);
		assert_eq!((cls.width, cls.height), (200, 228));
		assert_eq!(cls.cfgs, ["pbl_color", "pbl_platform_emery", "pbl_rect"]);
	}

	#[test]
	fn frozen_define_and_revision_must_agree() {
		let mut defines = MINIMAL.to_vec();
		defines.push("PBL_SDK_FROZEN");
		let err = classify(Platform::Emery, &upstream(&defines))
			.expect_err("PBL_SDK_FROZEN without FROZEN_AT_REVISION");
		assert!(err.to_string().contains("PBL_SDK_FROZEN"));
	}

	#[test]
	fn process_info_bits_are_unshifted() {
		let bindings = Platform::iter()
			.map(|p| {
				format!(
					"pub const PebbleProcessInfoFlags_PROCESS_INFO_PLATFORM_{}: \
					 PebbleProcessInfoFlags = {};",
					p.as_str().to_uppercase(),
					// Aplite is 1, in declaration order after that
					64 * (p as u32 + 1)
				)
			})
			.collect::<Vec<_>>()
			.join("\n");
		let bits = process_info_bits(&bindings).expect("well-formed bindings");
		assert_eq!(bits.len(), Platform::iter().count());
		assert!(bits.iter().all(|(_, bit)| (1..=7).contains(bit)));
	}
}
