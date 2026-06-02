use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use color_eyre::{Result, eyre::Context};
use pebble_types::Platform;

#[derive(Debug, Clone, clap::Parser)]
/// Create a new Pebble app project
pub struct NewProject {
    /// Name of your cargo package.
    name: String,
    /// Watch you want to target
    platform: Platform,
    /// Path to pebble-sys or gneiss
    /// TODO: Extend to allow git and cargo-version
    dep_path: PathBuf,
    /// Watchface or App?
    app_type: AppType,
    #[arg(long)]
    /// location to create project. Defaults to ./{name}
    path: Option<PathBuf>,
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
        self.cargo_toml(&path)
            .wrap_err("Failed to write Cargo.toml")?;
        self.cargo_config(&path)
            .wrap_err("Failed to write cargo/config.toml")?;
        //toolchain(&self, &path).wrap_err("Failed to write rust_toolchain.toml")?;
        self.main_rs(&path)
            .wrap_err("Failed to write src/main.rs")?;
        Ok(())
    }

    fn _toolchain(&self, path: &PathBuf) -> Result<()> {
        std::fs::write(path.join("rust-toolchain.toml"), "[toolchain]")
            .wrap_err("Can't write toolchian file")
    }

    fn cargo_config(&self, path: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(path.join(".cargo"))?;
        let build = PebbleBuild {
            target: self.platform.app_target_triple().to_string(),
        };
        let target = BTreeMap::from_iter(vec![(
            self.platform.app_target_triple().to_string(),
            BTreeMap::from_iter(vec![(
                "rustflags".to_string(),
                vec![
                    "-C",
                    "link-arg=-Tpebble_app.ld",
                    "-C",
                    "link-arg=--gc-sections",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            )]),
        )]);
        let conf = PebbleCargoConfig {
            build,
            target,
            unstable: BTreeMap::from_iter(vec![
                (
                    "build-std".to_string(),
                    vec!["core".to_string(), "alloc".to_string()],
                ),
                (
                    "build-std-features".to_string(),
                    vec!["panic_immediate_abort".to_string()],
                ),
            ]),
        };
        std::fs::write(path.join(".cargo/config.toml"), &toml::to_string(&conf)?)?;
        Ok(())
    }

    fn cargo_toml(&self, path: &Path) -> Result<()> {
        let metadata = Metadata {
            pebble: PebbleMetadata {
                uuid: Uuid::new_v4(),
                platform: self.platform,
                watchface: matches!(self.app_type, AppType::Watchface),
            },
        };
        let package = Package {
            name: self.name.to_string(),
            version: String::from("0.1.0"),
            edition: String::from("2024"),
            metadata,
        };
        let mut dependencies = BTreeMap::new();
        dependencies.insert(
            String::from(if self.sys_only {
                "pebble-sys"
            } else {
                "gneiss"
            }),
            Dependency::Path {
                path: self.dep_path.canonicalize()?,
                features: vec![self.platform.to_string()],
            },
        );
        let manifest = PebbleManifest {
            package,
            dependencies,
            profile: Profiles::default(),
        };
        std::fs::write(&path.join("Cargo.toml"), &toml::to_string(&manifest)?)?;

        Ok(())
    }

    fn main_rs(&self, path: &Path) -> Result<()> {
        let (includes, main): (syn::Stmt, syn::ItemFn) = if self.sys_only {
            (
                syn::parse_quote!(
                    use pebble_sys;
                ),
                syn::parse_quote! {

                    #[unsafe(no_mangle)]
                    pub extern "C" fn main() {
                        todo!("Write a cool watch app")
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
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum AppType {
    Watchface,
    Application,
}

#[derive(Debug, Clone, Serialize)]
pub struct PebbleCargoConfig {
    build: PebbleBuild,
    target: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    unstable: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PebbleBuild {
    target: String,
}

/// A generated watchapp `Cargo.toml`.
#[derive(Debug, Clone, Serialize)]
pub struct PebbleManifest {
    pub package: Package,
    pub dependencies: BTreeMap<String, Dependency>,
    pub profile: Profiles,
}

#[derive(Debug, Clone, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: String,
    /// Keep last: TOML requires scalar keys before sub-tables, and this emits
    /// `[package.metadata.pebble]`.
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    pub pebble: PebbleMetadata,
}

/// Contents of `[package.metadata.pebble]` — written by `new`, read back by
/// `build`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PebbleMetadata {
    pub uuid: Uuid,
    pub platform: Platform,
    #[serde(default)]
    pub watchface: bool,
}

/// A single `[dependencies]` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// `dep = "1.2.3"`
    Version {
        verison: String,
        features: Vec<String>,
    },
    /// `dep = { path = "..." }`
    Path {
        path: PathBuf,
        features: Vec<String>,
    },
    /// `dep = { git = "...", <rev|tag|branch> = "..." }`
    Git {
        git: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
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
    pub opt_level: String,
    pub lto: String,
    pub codegen_units: u32,
    pub panic: String,
    pub strip: String,
    pub overflow_checks: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            opt_level: "z".into(),
            lto: "fat".into(),
            codegen_units: 1,
            panic: "abort".into(),
            strip: "symbols".into(),
            overflow_checks: false,
        }
    }
}
