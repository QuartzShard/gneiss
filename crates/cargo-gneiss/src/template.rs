use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	process::Command,
};

use color_eyre::{Result, eyre::Context};
use gneiss_build::{MessageKey, PebbleMetadata, Resource, ResourceType};
use gneiss_types::Platform;
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
		std::fs::write(path.join(".gitignore"), "/target\n")
			.wrap_err("Failed to write .gitignore")?;
		std::fs::write(path.join("clippy.toml"), include_str!("clippy.toml"))
			.wrap_err("Failed to write clippy.toml")?;
		Ok(())
	}

	/// The app must build on nightly with `rust-src`: `-Zbuild-std` recompiles
	/// `core`/`alloc` (no prebuilt sysroot exists for the custom target), and
	/// `-Zjson-target-spec` is needed to consume the vendored `.json` at all.
	fn toolchain(path: &Path) -> Result<()> {
		std::fs::write(
			path.join("rust-toolchain.toml"),
			"[toolchain]\nchannel = \"nightly-2026-10-09\"\ncomponents = [\"rust-src\"]\n",
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
		.chain([
			"-Zfmt-debug=none",
			"-Zunstable-options",
			"-Cpanic=immediate-abort",
		])
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
		let gneiss_dep = Dependency::Version {
			version:  gneiss_build::RELEASE_TRAIN.to_string(),
			features: vec![],
		};
		let dependencies = BTreeMap::from_iter([("gneiss".to_string(), gneiss_dep.clone())]);
		let build_dependencies = BTreeMap::from_iter([("gneiss-build".to_string(), gneiss_dep)]);
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

			use gneiss::{
				prelude::*,
				ui::{
					Layer, Layout, StaticWindow, TextAlignment, TextLayer, UiGuard, UiQueue, Window,
					WindowCtx, graphics::{Point, Rectangle, Size},
				},
			};

			static MAIN: StaticWindow<Main> = StaticWindow::new();

			struct Main {
				_text: TextLayer,
			}

			impl Layout for Main {
				fn load(ctx: &mut WindowCtx<'_>) -> Self {
					let mut root = ctx.root();
					let bounds = root.bounds();
					let mut text = TextLayer::new(Rectangle::new(
						Point::new(0, (bounds.size.height as i32 - 40) / 2),
						Size::new(bounds.size.width, 40),
					))
					.pbl_expect(c"text layer");
					text.set_text(c"Write a cool watch app!")
						.set_alignment(TextAlignment::Center);
					root.add_child(&mut text);
					Self { _text: text }
				}
			}

			#[gneiss::entry]
			fn main(ui: &mut UiGuard) {
				MAIN.set(Window::new(None).pbl_expect(c"main window"));
				UiQueue::new(ui).push(&MAIN, false);
			}
		};
		// prettyplease drops the blank lines between items
		let mut source = prettyplease::unparse(&main);
		for item in [
			"extern ",
			"use ",
			"static ",
			"struct ",
			"impl ",
			"#[gneiss::entry]",
		] {
			source = source.replace(&format!("\n{item}"), &format!("\n\n{item}"));
		}
		std::fs::write(path.join("src/main.rs"), source).wrap_err("Cannot write main.rs")
	}

	fn build_rs(path: &Path) -> Result<()> {
		let build: syn::File = syn::parse_quote! {

			fn main() -> gneiss_build::color_eyre::eyre::Result<()> {
				gneiss_build::gen_includes()
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
}

/// `[unstable]` table. Heterogeneous value types (arrays + a bool), so it can't
/// be a plain map.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Unstable {
	build_std:          Vec<String>,
	build_std_features: Vec<String>,
	json_target_spec:   bool,
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
		version:  String,
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
	pub trim_paths:      String,
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
			trim_paths:      "all".into(),
		}
	}
}
