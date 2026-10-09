use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	process::Command,
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt},
};
use pebble_build::{MessageKey, PebbleMetadata, Resource, ResourceType};
use pebble_types::Platform;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn git_author() -> String {
	let cfg = |key: &str| -> Option<String> {
		let out = Command::new("git")
			.args(["config", "--get", key])
			.output()
			.ok()?;
		out.status.success().then_some(())?;
		let v = String::from_utf8(out.stdout).ok()?.trim().to_owned();
		(!v.is_empty()).then_some(v)
	};
	match (cfg("user.name"), cfg("user.email")) {
		(Some(n), _) => n,
		(None, Some(e)) => format!("<{e}>"),
		(None, None) => std::env::var("USER")
			.or_else(|_| std::env::var("USERNAME"))
			.unwrap_or_else(|_| "anonymous".into()),
	}
}

#[derive(Debug, Clone, clap::Parser)]
/// Create a new Pebble app project
pub struct NewProject {
	/// Name of your cargo package.
	name:     String,
	/// Watch(es) to target — repeat (`-p emery -p basalt`) or comma-separate.
	#[arg(short = 'p', long = "platform", required = true, value_delimiter = ',')]
	platform: Vec<Platform>,
	/// Path to gneiss
	/// TODO: Extend to allow git and cargo-version
	dep_path: PathBuf,
	/// Watchface or App?
	app_type: AppType,
	#[arg(long)]
	/// location to create project. Defaults to ./{name}
	path:     Option<PathBuf>,
}

impl NewProject {
	pub fn create(&self) -> Result<()> {
		let path: &Path = &self
			.path
			.clone()
			.unwrap_or_else(|| PathBuf::from(format!("./{}", self.name)));
		println!(
			"Creating {} in {} for {:?}...",
			self.name,
			path.display(),
			self.platform
		);
		std::fs::create_dir_all(path.join("src"))?;
		std::fs::create_dir_all(path.join("resources"))?;
		std::fs::write(
			path.join("resources/icon.png"),
			include_bytes!("../default_icon.png"),
		)?;
		self.cargo_toml(path)
			.wrap_err("Failed to write Cargo.toml")?;
		self.cargo_config(path)
			.wrap_err("Failed to write cargo/config.toml")?;
		Self::toolchain(path).wrap_err("Failed to write rust-toolchain.toml")?;
		Self::main_rs(path).wrap_err("Failed to write src/main.rs")?;
		Self::build_rs(path).wrap_err("Failed to write build.rs")?;
		Ok(())
	}

	/// The app must build on nightly with `rust-src`: `-Zbuild-std` recompiles
	/// `core`/`alloc` (no prebuilt sysroot exists for the custom target), and
	/// `-Zjson-target-spec` is needed to consume the vendored `.json` at all.
	fn toolchain(path: &Path) -> Result<()> {
		std::fs::write(
			path.join("rust-toolchain.toml"),
			"[toolchain]\nchannel = \"nightly\"\ncomponents = [\"rust-src\"]\n",
		)
		.wrap_err("Can't write toolchain file")
	}

	fn cargo_config(&self, path: &Path) -> Result<()> {
		std::fs::create_dir_all(path.join(".cargo"))?;

		let triple = self.platform[0].app_target_triple();
		std::fs::write(
			path.join(format!("{triple}.json")),
			include_str!("../thumbv7m-pebble.json"),
		)?;

		let build = PebbleBuild {
			target: format!("{triple}.json"),
		};
		// `relocation-model=pic` lives in the target spec (so build-std picks it
		// up for core/alloc); only link-time flags belong here.
		let rustflags = vec![
			"link-arg=-Tpebble_app.ld",
			"link-arg=--gc-sections",
			"link-arg=--emit-relocs",
		]
		.into_iter()
		.flat_map(|f| ["-C", f])
		.chain(["-Zfmt-debug=none"])
		.map(String::from)
		.collect();
		let target = BTreeMap::from_iter(vec![(
			triple.to_string(),
			BTreeMap::from_iter(vec![("rustflags".to_string(), rustflags)]),
		)]);
		let conf = PebbleCargoConfig {
			build,
			target,
			unstable: Unstable {
				build_std:          vec!["core".to_string(), "alloc".to_string()],
				// core's str/slice comparisons lower to C `memcmp`, which nothing
				// on the bare-metal link provides unless compiler_builtins is
				// built with its `mem` feature.
				build_std_features: vec![
					"compiler-builtins-mem".to_string(),
					"optimize_for_size".to_string(),
				],
				json_target_spec:   true,
				trim_paths:         true,
			},
			profile: ConfigProfiles {
				release: ConfigProfile {
					trim_paths: "all".to_string(),
				},
			},
		};
		std::fs::write(path.join(".cargo/config.toml"), &toml::to_string(&conf)?)?;
		Ok(())
	}

	fn cargo_toml(&self, path: &Path) -> Result<()> {
		let resources = BTreeMap::from_iter(vec![("APP_ICON".to_string(), Resource {
			file:             "icon.png".into(),
			target_platforms: self.platform.clone(),
			res:              ResourceType::Icon,
		})]);
		let message_keys = Some(vec![
			MessageKey::Single("example_key".to_string()),
			MessageKey::Array {
				name: "example_buf".to_string(),
				len:  16,
			},
		]);
		let metadata = Metadata {
			pebble: PebbleMetadata {
				uuid: Uuid::new_v4(),
				watchface: matches!(self.app_type, AppType::Watchface),
				resources,
				message_keys,
			},
		};
		let package = Package {
			name: self.name.clone(),
			authors: vec![git_author()],
			version: String::from("0.1.0"),
			edition: String::from("2024"),
			metadata,
		};
		let dep_path = self.dep_path.canonicalize()?;
		let crates_dir = dep_path.parent().ok_or_eyre("no parent")?;
		let mut dependencies = BTreeMap::new();
		dependencies.insert("gneiss".to_string(), Dependency::Path {
			path:     crates_dir.join("gneiss"),
			features: vec![],
		});
		let build_dependencies =
			BTreeMap::from_iter([("pebble-build".to_string(), Dependency::Path {
				path:     crates_dir.join("pebble-build"),
				features: vec![],
			})]);
		let features: BTreeMap<String, Vec<String>> = self
			.platform
			.iter()
			.map(|p| (p.to_string(), vec![format!("gneiss/{p}")]))
			.collect();
		let manifest = PebbleCargoManifest {
			package,
			dependencies,
			build_dependencies,
			features,
			profile: Profiles::default(),
		};
		let mut doc = toml::to_string(&manifest)?.parse::<toml_edit::DocumentMut>()?;
		let pkg = doc["package"]
			.as_table_mut()
			.expect("manifest was just serialized from PebbleCargoManifest above");
		let mut authors = pkg.key_mut("authors").expect("authors key");
		let decor = authors.leaf_decor_mut();
		let prev = decor
			.prefix()
			.and_then(|p| p.as_str())
			.unwrap_or("")
			.to_string();
		decor.set_prefix(format!(
			"{prev}# authors[0] used as the watch-menu \"company\"\n"
		));

		std::fs::write(path.join("Cargo.toml"), doc.to_string())?;

		Ok(())
	}

	fn main_rs(path: &Path) -> Result<()> {
		let main: syn::File = syn::parse_quote! {
			#![no_std]
			#![no_main]

			extern crate alloc;

			use gneiss::sys;

			#[gneiss::entry]
			fn main() {
				unsafe {
					let window = sys::window_create();
					let root = sys::window_get_root_layer(window);
					let bounds = sys::layer_get_bounds(root);

					let text_layer = sys::text_layer_create(sys::GRect {
						origin: sys::GPoint { x: 0, y: (bounds.size.h - 40) / 2 },
						size:   sys::GSize { w: bounds.size.w, h: 40 },
					});
					sys::text_layer_set_text(text_layer, c"Write a cool watch app!".as_ptr());
					sys::text_layer_set_text_alignment(
						text_layer,
						sys::GTextAlignment_GTextAlignmentCenter,
					);
					sys::layer_add_child(root, sys::text_layer_get_layer(text_layer));

					sys::window_stack_push(window, false);
				}
			}
		};
		std::fs::write(path.join("src/main.rs"), prettyplease::unparse(&main))
			.wrap_err("Cannot write main.rs")
	}

	fn build_rs(path: &Path) -> Result<()> {
		let build: syn::File = syn::parse_quote! {

			fn main() -> pebble_build::color_eyre::eyre::Result<()> {
				pebble_build::gen_includes()
			}
		};
		std::fs::write(path.join("build.rs"), prettyplease::unparse(&build))
			.wrap_err("Cannot write build.rs")
	}
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum AppType {
	Watchface,
	Application,
}

#[derive(Debug, Clone, Serialize)]
pub struct PebbleCargoConfig {
	build:    PebbleBuild,
	target:   BTreeMap<String, BTreeMap<String, Vec<String>>>,
	unstable: Unstable,
	profile:  ConfigProfiles,
}

/// `[unstable]` table. Heterogeneous value types (arrays + a bool), so it can't
/// be a plain map.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Unstable {
	build_std:          Vec<String>,
	build_std_features: Vec<String>,
	json_target_spec:   bool,
	trim_paths:         bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigProfiles {
	release: ConfigProfile,
}

/// Profile settings that need `[unstable]` flags, so they live in the cargo config rather than the
/// manifest.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigProfile {
	trim_paths: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PebbleBuild {
	target: String,
}

/// A generated watchapp `Cargo.toml`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PebbleCargoManifest {
	pub package:            Package,
	pub dependencies:       BTreeMap<String, Dependency>,
	pub build_dependencies: BTreeMap<String, Dependency>,
	/// `[features]` — one entry per platform, each forwarding to the SDK dep.
	pub features:           BTreeMap<String, Vec<String>>,
	pub profile:            Profiles,
}

#[derive(Debug, Clone, Serialize)]
pub struct Package {
	pub name:     String,
	/// The build step reads the first entry back as the Pebble manifest company name
	pub authors:  Vec<String>,
	pub version:  String,
	pub edition:  String,
	/// Keep last: TOML requires scalar keys before sub-tables, and this emits
	/// `[package.metadata.pebble]`.
	pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
	pub pebble: PebbleMetadata,
}

/// A single `[dependencies]` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
	/// `dep = "1.2.3"`
	Version {
		verison:  String,
		features: Vec<String>,
	},
	/// `dep = { path = "..." }`
	Path {
		path:     PathBuf,
		features: Vec<String>,
	},
	/// `dep = { git = "...", <rev|tag|branch> = "..." }`
	Git {
		git:      String,
		#[serde(skip_serializing_if = "Option::is_none")]
		rev:      Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		tag:      Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		branch:   Option<String>,
		features: Vec<String>,
	},
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Profiles {
	pub release: Profile,
}

/// The watchapp's `[profile.release]`. The generated app is a standalone
/// package, so it carries its own firmware profile rather than inheriting one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Profile {
	pub opt_level:       String,
	pub lto:             String,
	pub codegen_units:   u32,
	pub panic:           String,
	pub strip:           String,
	pub overflow_checks: bool,
}

impl Default for Profile {
	fn default() -> Self {
		Self {
			opt_level:       "z".into(),
			lto:             "fat".into(),
			codegen_units:   1,
			panic:           "abort".into(),
			// The intermediate ELF must keep its symbol table and relocations:
			// the packager reads `main`/`pbl_table_addr` via nm and harvests
			// `.rel.data`/`.got`, and `--strip-all` is rejected alongside
			// `--emit-relocs`. The shipped `.bin` carries no symbols regardless.
			strip:           "none".into(),
			overflow_checks: false,
		}
	}
}
