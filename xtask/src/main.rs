use std::{
	path::{Path, PathBuf},
	process::Command,
};

use clap::Parser;
use color_eyre::{Result, eyre::bail};
use gneiss_types::Platform;
use strum::IntoEnumIterator;
use tempfile::TempDir;

use crate::{map_shim::map_bindgen_to_shimmed, platform_table::PlatformTable};

mod doxygen;
mod exported_symbols;
mod map_shim;
mod platform_table;

#[derive(clap::Parser)]
enum TaskArgs {
	RegenSys,
	SnapshotVendor {
		repo: PathBuf,
	},
	/// Check + clippy across the workspace: watch crates on the app target
	/// (emery), host crates on the host. Also compiles gneiss's doc examples, and checks the
	/// library crates share a release train.
	Ci,
}

fn main() -> Result<()> {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.filter_module("bindgen", log::LevelFilter::Error)
		.parse_default_env()
		.init();
	let _ = color_eyre::install();
	let args = TaskArgs::parse();
	match args {
		TaskArgs::RegenSys => regen_sys(),
		TaskArgs::SnapshotVendor { repo } => {
			snapshot_vendor(&repo.canonicalize()?)?;
			regen_sys()
		}
		TaskArgs::Ci => ci(),
	}
}

/// Check + clippy across the whole workspace: on-watch crates against the app
/// target (emery), host crates against the host. Runs every step and reports
/// which failed, so one break doesn't hide the rest.
fn ci() -> Result<()> {
	let root = workspace_root();
	let watch: &[&str] = &[
		"-p",
		"gneiss-types",
		"-p",
		"gneiss-sys",
		"-p",
		"gneiss",
		"--features",
		"gneiss/emery",
		"--target",
		"crates/cargo-gneiss/thumbv7m-pebble.json",
		"-Zbuild-std=core,alloc",
		"-Zjson-target-spec",
	];
	let host: &[&str] = &["-p", "cargo-gneiss", "-p", "gneiss-build", "-p", "xtask"];
	let doctests: &[&str] = &[
		"--doc",
		"-p",
		"gneiss",
		"--features",
		"gneiss/emery,gneiss/__doctest",
		"--target",
		"crates/cargo-gneiss/thumbv7m-pebble.json",
		"-Zbuild-std=core,alloc",
		"-Zjson-target-spec",
	];

	let steps: [(&str, &str, &[&str]); 5] = [
		("check", "watch", watch),
		("check", "host", host),
		("clippy", "watch", watch),
		("clippy", "host", host),
		("test", "doctests", doctests),
	];

	let mut failed = Vec::new();
	log::info!("release train");
	if let Err(e) = check_release_train(root) {
		log::error!("{e}");
		failed.push("release train".to_string());
	}
	for (cmd, side, args) in steps {
		log::info!("cargo {cmd} ({side})");
		let ok = Command::new("cargo")
			.arg(cmd)
			.args(args)
			.current_dir(root)
			.status()?
			.success();
		if !ok {
			failed.push(format!("{cmd} ({side})"));
		}
	}

	if !failed.is_empty() {
		bail!("ci failed: {}", failed.join(", "));
	}
	Ok(())
}

/// The library crates released in step on one major.minor (`gneiss_build::RELEASE_TRAIN`).
/// cargo-gneiss is versioned on its own.
const RELEASE_TRAIN: [&str; 5] = [
	"gneiss",
	"gneiss-macros",
	"gneiss-sys",
	"gneiss-types",
	"gneiss-build",
];

/// Fail unless every crate in [`RELEASE_TRAIN`] is on the same major.minor.
fn check_release_train(root: &Path) -> Result<()> {
	let out = Command::new("cargo")
		.args(["metadata", "--no-deps", "--format-version", "1"])
		.current_dir(root)
		.output()?;
	if !out.status.success() {
		bail!("cargo metadata failed");
	}
	let meta: serde_json::Value = serde_json::from_slice(&out.stdout)?;
	let trains: Vec<(&str, String)> = meta["packages"]
		.as_array()
		.into_iter()
		.flatten()
		.filter_map(|pkg| {
			let name = pkg["name"].as_str()?;
			let version = pkg["version"].as_str()?;
			RELEASE_TRAIN.contains(&name).then(|| {
				let train = version.split('.').take(2).collect::<Vec<_>>().join(".");
				(name, train)
			})
		})
		.collect();
	if trains.len() != RELEASE_TRAIN.len() {
		bail!("release train: expected {RELEASE_TRAIN:?} in the workspace, found {trains:?}");
	}
	if trains.iter().any(|(_, train)| *train != trains[0].1) {
		let listed: Vec<String> = trains
			.iter()
			.map(|(name, train)| format!("{name} {train}"))
			.collect();
		bail!(
			"release train: library crates must share a major.minor, found {}",
			listed.join(", ")
		);
	}
	Ok(())
}

/// Invoke bindgen on pebble.h, then map declarations -> shims to remove pebble.a runtime dep
fn regen_sys() -> Result<()> {
	// pebble.h #includes a libc surface plus two per-app generated headers that
	// do not exist at SDK-snapshot time.
	const STUBS: &[(&str, &str)] = &[
		// per-app generated; empty at SDK time. cargo-gneiss emits the real
		// ones per gneiss-app, exactly as `pebble build` does for C apps.
		("message_keys.auto.h", "#pragma once\n"),
		("src/resource_ids.auto.h", "#pragma once\n"),
		// libc surface. time.h provides time_t but deliberately NOT struct tm —
		// pebble.h defines its own; a second definition is the original bug.
		("time.h", "#pragma once\ntypedef long time_t;\n"),
		// pebble.h takes `size_t` parameters but only `#include`s <stdlib.h>;
		// pull stddef.h (clang freestanding, reachable via -isystem) so size_t
		// is defined for every platform. emery limped along because xsffi.h
		// happened to include stddef.h transitively; other platforms don't.
		// The allocator family has no applib prototype (the SDK exports it via
		// `implName` aliasing onto `task_malloc` &c.), so declare it here for
		// bindgen; the shim carve-out in map_shim un-skips these slots.
		(
			"stdlib.h",
			"#pragma once\n#include <stddef.h>\n\
			 void *malloc(size_t);\nvoid free(void *);\nvoid *realloc(void *, size_t);\n",
		),
		("stdio.h", "#pragma once\n"),
		// Like the allocator family, mem* is `skipDefinition` but exported; gneiss routes
		// compiler_builtins' mem* through these
		(
			"string.h",
			"#pragma once\n#include <stddef.h>\n\
			 void *memcpy(void *, const void *, size_t);\n\
			 void *memmove(void *, const void *, size_t);\n\
			 void *memset(void *, int, size_t);\n",
		),
		("locale.h", "#pragma once\n"),
	];
	let vendor = workspace_root().join("crates/gneiss-sys/vendor");
	// Check we have a vendor snapshot
	if !std::fs::exists(vendor.join("UPSTREAM"))? {
		bail!("No vendor snapshot, run `cargo xtask snapshot-vendor` to generate one")
	}
	let platforms = PlatformTable::load(&vendor)?;

	let resource_inc = clang_resource_dir()?.join("include").display().to_string();
	let clang_args: Vec<String> = [
		"--target=arm-none-eabi",
		"-std=c99",
		"-mthumb",
		"-mcpu=cortex-m3",
		"-nostdinc",
		"-fshort-enums",
		// Must preceed resource_inc
		"-isystem",
		&resource_inc,
		"-DRELEASE",
		"-DPBL_SDK_3",
	]
	.map(String::from)
	.into();
	for platform in Platform::iter() {
		let plat_vendor = vendor.join(platform.to_string());
		for (rel, contents) in STUBS {
			let path = plat_vendor.join(rel);
			std::fs::create_dir_all(path.parent().expect("vendor root exists"))?;
			std::fs::write(path, contents)?;
		}
		// Mirror the `pebble build` clang environment:
		//  --target=arm-none-eabi : 32-bit ARM EABI. Host x86_64
		//  -nostdinc              : host glibc is never consulted at all.
		//  -isystem <resource>    : clang's OWN freestanding builtins (<stdint.h>,
		//      <stdbool.h>, <stddef.h>, <stdarg.h>)
		//  -I <plat_vendor>       : resolves the angle-include libc stubs above and
		//      pebble.h's generated sibling headers (per-platform snapshot).
		let plat_vendor_arg = plat_vendor.display().to_string();
		let mut clang_args: Vec<String> = clang_args.clone();
		clang_args.extend_from_slice(&["-I", &plat_vendor_arg].map(String::from));
		clang_args.extend(platforms.clang_args(platform)?);

		let bindings = bindgen::builder()
			.header(plat_vendor.join("pebble.h").display().to_string())
			// We're not interested in JS FFI
			.blocklist_file(".*xsffi\\.h")
			.use_core()
			.generate_cstr(true)
			.clang_args(clang_args)
			.parse_callbacks(Box::new(doxygen::DoxygenComments))
			.generate()?;

		let mut mapped = map_bindgen_to_shimmed(&bindings, &vendor, platform)?;
		mapped.attrs = vec![
			syn::parse_quote!(#![doc = " SPDX-License-Identifier: Apache-2.0"]),
			syn::parse_quote!(#![doc = " This module is generated from `pebble.h` by bindgen + a shim-mapping step in the `regen-sys` xtask. Do not hand-edit"]),
		];
		// lib.rs uses `#[cfg_attr(feature = "<plat>", path = "<plat>.rs")]` to
		// select the per-platform snapshot at compile time. Each platform gets its
		// own checked-in file so multiple platforms can coexist in the tree.
		let out = workspace_root().join(format!("crates/gneiss-sys/src/platform/{platform}.rs"));
		std::fs::write(&out, prettyplease::unparse(&mapped))?;
		rustfmt(&out)?;
	}

	// pebble_process_info
	let mut bindings = bindgen::builder()
		.header(
			vendor
				// It's the same on every platform, so just grab one
				.join("emery/pebble_process_info.h")
				.display()
				.to_string(),
		)
		.use_core()
		.clang_args(clang_args)
		.generate_cstr(true)
		.blocklist_function("version_compare")
		.parse_callbacks(Box::new(doxygen::DoxygenComments))
		.generate()?
		.to_string();
	bindings = format!(
		"//! SPDX-License-Identifier: Apache-2.0\n//! Generated from pebble_process_info.h, do not hand-edit\n#![allow(warnings, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]\n{bindings}"
	);
	bindings = bindings.replace(
		"#[derive(Debug, Copy, Clone)]",
		"#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]\n#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]",
	);
	let process_info = workspace_root().join("crates/gneiss-types/src/pebble_process_info.rs");
	std::fs::write(&process_info, &bindings)?;
	rustfmt(&process_info)?;

	let bits = platform_table::process_info_bits(&bindings)?;
	let sdk = platform_table::sdk_versions(&vendor)?;
	let config = workspace_root().join("crates/gneiss-types/src/config.rs");
	std::fs::write(&config, platform_table::render(&platforms, &bits, &sdk)?)?;
	rustfmt(&config)?;
	Ok(())
}

/// Format a generated file with the workspace's rustfmt config.
fn rustfmt(path: &Path) -> Result<()> {
	let status = Command::new("rustfmt")
		// `.rustfmt.toml` sets nightly-only options; run from the workspace so
		// rustup picks the pinned toolchain
		.current_dir(workspace_root())
		.arg("--edition")
		.arg("2024")
		.arg(path)
		.status()?;
	if !status.success() {
		bail!("rustfmt failed on {}", path.display());
	}
	Ok(())
}

/// Grab the necessary files to build the sys crate from coredevices/PebbleOS. Involves running
/// `generate_pebble_native_sdk_files.py`
fn snapshot_vendor(repo: &Path) -> Result<()> {
	check_pebbleos_present(repo)?;

	let config: PathBuf = repo.join("tools/generate_native_sdk/exported_symbols.json");
	let tmp = TempDir::new_in(".")?;
	log::info!("Generating pebble.h in {}...", tmp.path().display());
	let vendor = workspace_root().join("crates/gneiss-sys/vendor");
	std::fs::create_dir_all(&vendor)?;

	log::info!("Looking for `uv`...");
	if !Command::new("which").arg("uv").status()?.success() {
		bail!("`uv` not found on your $PATH, you probably want to install pebble-tool first")
	}

	for platform in Platform::iter() {
		log::info!("Building SDK headers for {platform}...");
		// build_sdk.py nests `<platform>/` under `--output-dir` itself, and
		// makes its own `include/`, `lib/`, and `src/fw/` subdirs (line 94-96).
		// So we just hand it the tmp root and let it lay out tmp/<platform>/…
		// and tmp/src/… underneath.
		let codegen = Command::new("uv")
			.current_dir(repo)
			.arg("run")
			.arg("--with")
			.arg("libclang==18.*")
			.arg("tools/build_sdk.py")
			.arg(platform.to_string())
			.arg("--output-dir")
			.arg(tmp.path())
			.output()?;
		if !codegen.status.success() {
			bail!(
				"Generator for {platform} failed: {}",
				String::from_utf8_lossy(&codegen.stderr)
			)
		}
		let tmp_include = tmp.path().join(platform.to_string()).join("include");
		let dest = vendor.join(platform.to_string());
		let _ = std::fs::remove_dir_all(&dest);
		std::fs::rename(&tmp_include, &dest)?;
	}

	std::fs::copy(config, vendor.join("exported_symbols.json"))?;

	// `tools/pebble_sdk_platform.py` imports nothing, so python can dump it as
	// JSON; `regen-sys` reads that into `gneiss-types`'s config table
	log::info!("Dumping upstream's platform table...");
	let dump = Command::new("uv")
		.current_dir(repo)
		.args(["run", "python", "-c"])
		.arg(
			"import json, sys; sys.path.insert(0, 'tools'); import pebble_sdk_platform as p; \
			 json.dump(p.pebble_platforms, sys.stdout, indent=2, sort_keys=True)",
		)
		.output()?;
	if !dump.status.success() {
		bail!(
			"Dumping pebble_sdk_platform.py failed: {}",
			String::from_utf8_lossy(&dump.stderr)
		)
	}
	std::fs::write(vendor.join("platforms.json"), &dump.stdout)?;

	// Get a commit hash from the repo...
	let hash = Command::new("git")
		.current_dir(repo)
		.arg("rev-parse")
		.arg("HEAD")
		.output()
		.map_or("NotAGitRepo".into(), |output| {
			String::from_utf8_lossy(&output.stdout).into_owned()
		});
	// ...and write out a fingerprint file
	let now = chrono::Local::now().to_rfc3339();
	std::fs::write(
		vendor.join("UPSTREAM"),
		format!(
			"Source: https://github.com/coredevices/PebbleOS\n\
             License: Apache-2.0 (Copyright Google LLC) — see LICENSE.apache-2.0\n\
             PebbleOS Commit: {}\nSnapshot made: {}",
			hash.trim(),
			now
		),
	)?;
	log::info!("Done! See crates/gneiss-sys/vendor");
	Ok(())
}

/// Check for specifically the files we need to build our snapshot - which probably means the repo
/// is coredevices/PebbleOS
fn check_pebbleos_present(repo: &Path) -> Result<()> {
	let required = [
		"tools/generate_native_sdk/generate_pebble_native_sdk_files.py",
		"tools/generate_native_sdk/exported_symbols.json",
		"tools/pebble_sdk_platform.py",
		"src",
	];
	let submodule = [
		// FreeRTOS headers go into the clang flags; parsing fails without them
		"third_party/freertos/FreeRTOS-Kernel/FreeRTOS/Source/include/FreeRTOS.h",
		// emery has HAS_MODDABLE_XS → generator unconditionally shutil.copy's this;
		// its absence is a hard FileNotFoundError crash, not a soft failure
		"third_party/moddable/moddable/xs/includes/xsffi.h",
	];
	for rel in required {
		let p = repo.join(rel);
		if !p.exists() {
			bail!(
				"{} not found — is {} a PebbleOS checkout?",
				p.display(),
				repo.display()
			);
		}
	}
	for rel in submodule {
		let p = repo.join(rel);
		if !p.exists() {
			bail!(
				"{} not found — ensure git submodule is initialised",
				p.display(),
			);
		}
	}
	Ok(())
}

fn workspace_root() -> &'static Path {
	// xtask is always <workspace>/xtask; its parent is the workspace root.
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("xtask crate is <workspace>/xtask")
}

fn clang_resource_dir() -> Result<PathBuf> {
	let out = Command::new("clang").arg("-print-resource-dir").output()?;
	if !out.status.success() {
		bail!("`clang -print-resource-dir` failed — is clang on PATH?");
	}
	Ok(PathBuf::from(
		String::from_utf8_lossy(&out.stdout).trim().to_string(),
	))
}
