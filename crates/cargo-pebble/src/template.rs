use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	process::Command,
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt},
};
use pebble_build::{PebbleMetadata, Resource, ResourceType};
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
	/// Path to pebble-sys or gneiss
	/// TODO: Extend to allow git and cargo-version
	dep_path: PathBuf,
	/// Watchface or App?
	app_type: AppType,
	#[arg(long)]
	/// location to create project. Defaults to ./{name}
	path:     Option<PathBuf>,
	/// Pull in `pebble-sys` only, no `gneiss`
	#[arg(long)]
	sys_only: bool,
}

impl NewProject {
	pub fn create(&self) -> Result<()> {
		let path = self
			.path
			.as_ref()
			.cloned()
			.unwrap_or(PathBuf::from(format!("./{}", self.name)));
		println!(
			"Creating {} in {} for {:?}...",
			self.name,
			path.display(),
			self.platform
		);
		std::fs::create_dir_all(&path.join("src"))?;
		std::fs::create_dir_all(&path.join("resources"))?;
		self.cargo_toml(&path)
			.wrap_err("Failed to write Cargo.toml")?;
		self.cargo_config(&path)
			.wrap_err("Failed to write cargo/config.toml")?;
		self.toolchain(&path)
			.wrap_err("Failed to write rust-toolchain.toml")?;
		self.main_rs(&path)
			.wrap_err("Failed to write src/main.rs")?;
		self.build_rs(&path).wrap_err("Failed to write build.rs")?;
		Ok(())
	}

	/// The app must build on nightly with `rust-src`: `-Zbuild-std` recompiles
	/// `core`/`alloc` (no prebuilt sysroot exists for the custom target), and
	/// `-Zjson-target-spec` is needed to consume the vendored `.json` at all.
	fn toolchain(&self, path: &PathBuf) -> Result<()> {
		std::fs::write(
			path.join("rust-toolchain.toml"),
			"[toolchain]\nchannel = \"nightly\"\ncomponents = [\"rust-src\"]\n",
		)
		.wrap_err("Can't write toolchain file")
	}

	fn cargo_config(&self, path: &PathBuf) -> Result<()> {
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
				build_std:        vec!["core".to_string(), "alloc".to_string()],
				json_target_spec: true,
			},
		};
		std::fs::write(path.join(".cargo/config.toml"), &toml::to_string(&conf)?)?;
		Ok(())
	}

	fn cargo_toml(&self, path: &Path) -> Result<()> {
		let resources = BTreeMap::from_iter(vec![("EXAMPLE".to_string(), Resource {
			file:             "data.bin".into(),
			target_platforms: vec![Platform::Emery],
			res_type:         ResourceType::Raw,
		})]);
		let metadata = Metadata {
			pebble: PebbleMetadata {
				uuid: Uuid::new_v4(),
				watchface: matches!(self.app_type, AppType::Watchface),
				resources,
			},
		};
		let package = Package {
			name: self.name.to_string(),
			authors: vec![git_author()],
			version: String::from("0.1.0"),
			edition: String::from("2024"),
			metadata,
		};
		let dep_name = if self.sys_only {
			"pebble-sys"
		} else {
			"gneiss"
		};
		let mut dependencies = BTreeMap::new();
		dependencies.insert(dep_name.to_string(), Dependency::Path {
			path:     self.dep_path.canonicalize()?,
			features: vec![],
		});
		let build_dependencies =
			BTreeMap::from_iter([("pebble-build".to_string(), Dependency::Path {
				path:     self
					.dep_path
					.parent()
					.ok_or_eyre("no parent")?
					.join("pebble-build"),
				features: vec![],
			})]);
		// One feature per target platform, each forwarding to the SDK dep, so
		// `cargo build --features emery` selects the board.
		let features: BTreeMap<String, Vec<String>> = self
			.platform
			.iter()
			.map(|p| (p.to_string(), vec![format!("{dep_name}/{p}")]))
			.collect();
		let manifest = PebbleCargoManifest {
			package,
			dependencies,
			build_dependencies,
			features,
			profile: Profiles::default(),
		};
		let mut doc = toml::to_string(&manifest)?.parse::<toml_edit::DocumentMut>()?;
		let pkg = &mut doc["package"].as_table_mut().unwrap();
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

		std::fs::write(&path.join("Cargo.toml"), doc.to_string())?;

		Ok(())
	}

	fn main_rs(&self, path: &Path) -> Result<()> {
		let (includes, main): (proc_macro2::TokenStream, syn::ItemFn) = if self.sys_only {
			(
				syn::parse_quote!(
					use pebble_sys::*;
					mod resources {
						include!(concat!(env!("OUT_DIR"), "/resources.rs"));
					}
				),
				syn::parse_quote! {

					#[unsafe(no_mangle)]
					pub extern "C" fn main() {
						unsafe {
							let window = window_create();
							let root = window_get_root_layer(window);
							let bounds = layer_get_bounds(root);

							let text_layer = text_layer_create(GRect {
								origin: GPoint { x: 0, y: (bounds.size.h - 40) / 2 },
								size:   GSize { w: bounds.size.w, h: 40 },
							});
							text_layer_set_text(text_layer, c"Write a cool watch app!".as_ptr());
							text_layer_set_text_alignment(
								text_layer,
								GTextAlignment_GTextAlignmentCenter,
							);
							layer_add_child(root, text_layer_get_layer(text_layer));

							window_stack_push(window, false);
							app_event_loop();
						}
					}
				},
			)
		} else {
			(
				syn::parse_quote!(
					use gneiss;
				),
				syn::parse_quote! {
					#[gneiss::entry]
					fn main() {
						todo!("Write a cool watch app")
					}
				},
			)
		};
		let main: syn::File = syn::parse_quote! {
			#![no_std]
			#![no_main]

			#[panic_handler]
			fn pbl_panic(_info: &core::panic::PanicInfo<'_>) -> ! {
				unsafe { core::arch::asm!("udf 0") };
				loop {}
			}

			#includes

			#main
		};
		std::fs::write(path.join("src/main.rs"), prettyplease::unparse(&main))
			.wrap_err("Cannot write main.rs")
	}

	fn build_rs(&self, path: &Path) -> Result<()> {
		let build: syn::File = syn::parse_quote! {
			use pebble_build;

			fn main() {
				pebble_build::gen_resource_declarations().expect("Resource codegen failure");
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
	build_std:        Vec<String>,
	json_target_spec: bool,
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
