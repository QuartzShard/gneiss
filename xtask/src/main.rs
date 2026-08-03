use std::{
	path::{Path, PathBuf},
	process::Command,
};

use clap::Parser;
use color_eyre::{Result, eyre::bail};
use pebble_types::Platform;
use strum::IntoEnumIterator;
use tempfile::TempDir;

use crate::map_shim::map_bindgen_to_shimmed;

mod exported_symbols;
mod map_shim;

#[derive(clap::Parser)]
enum TaskArgs {
	RegenSys,
	SnapshotVendor { repo: PathBuf },
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
	}
}

/// Invoke bindgen on pebble.h, then map declarations -> shims to remove pebble.a runtime dep
fn regen_sys() -> Result<()> {
	// pebble.h #includes a libc surface plus two per-app generated headers that
	// do not exist at SDK-snapshot time.
	const STUBS: &[(&str, &str)] = &[
		// per-app generated; empty at SDK time. cargo-pebble emits the real
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
		("stdlib.h", "#pragma once\n#include <stddef.h>\n"),
		("stdio.h", "#pragma once\n"),
		("string.h", "#pragma once\n"),
		("locale.h", "#pragma once\n"),
	];
	let vendor = workspace_root().join("crates/pebble-sys/vendor");
	// Check we have a vendor snapshot
	if !std::fs::exists(vendor.join("UPSTREAM"))? {
		bail!("No vendor snapshot, run `cargo xtask snapshot-vendor` to generate one")
	}

	let resource_inc = clang_resource_dir()?.join("include").display().to_string();
	let clang_args: Vec<String> = [
		"--target=arm-none-eabi",
		"-std=c99",
		"-mthumb",
		"-mcpu=cortex-m3",
		"-nostdinc",
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
		clang_args.extend(clang_args_for(platform));

		let bindings = bindgen::builder()
			.header(plat_vendor.join("pebble.h").display().to_string())
			// We're not interested in JS FFI
			.blocklist_file(".*xsffi\\.h")
			.use_core()
			.generate_cstr(true)
			.clang_args(clang_args)
			.generate()?;

		let mut mapped = map_bindgen_to_shimmed(&bindings, &vendor, platform)?;
		mapped.attrs = vec![
			syn::parse_quote!(#![doc = " SPDX-License-Identifier: Apache-2.0"]),
			syn::parse_quote!(#![doc = " This module is generated from `pebble.h` by bindgen + a shim-mapping step in the `regen-sys` xtask. Do not hand-edit"]),
		];
		// lib.rs uses `#[cfg_attr(feature = "<plat>", path = "<plat>.rs")]` to
		// select the per-platform snapshot at compile time. Each platform gets its
		// own checked-in file so multiple platforms can coexist in the tree.
		let out = workspace_root().join(format!("crates/pebble-sys/src/platform/{platform}.rs"));
		std::fs::write(&out, prettyplease::unparse(&mapped))?;
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
		.generate()?
		.to_string();
	bindings = format!(
		"//! SPDX-License-Identifier: Apache-2.0\n//! Generated from pebble_process_info.h, do not hand-edit\n#![allow(warnings, clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]\n{bindings}"
	);
	bindings = bindings.replace(
		"#[derive(Debug, Copy, Clone)]",
		"#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]\n#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]",
	);
	std::fs::write(
		workspace_root().join("crates/pebble-types/src/pebble_process_info.rs"),
		&bindings,
	)?;
	Ok(())
}

/// Grab the necessary files to build the sys crate from coredevices/PebbleOS. Involves running
/// `generate_pebble_native_sdk_files.py`
fn snapshot_vendor(repo: &Path) -> Result<()> {
	check_pebbleos_present(repo)?;

	let config: PathBuf = repo.join("tools/generate_native_sdk/exported_symbols.json");
	let tmp = TempDir::new_in(".")?;
	log::info!("Generating pebble.h in {}...", tmp.path().display());
	let vendor = workspace_root().join("crates/pebble-sys/vendor");
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
	log::info!("Done! See crates/pebble-sys/vendor");
	Ok(())
}

/// Check for specifically the files we need to build our snapshot - which probably means the repo
/// is coredevices/PebbleOS
fn check_pebbleos_present(repo: &Path) -> Result<()> {
	let required = [
		"tools/generate_native_sdk/generate_pebble_native_sdk_files.py",
		"tools/generate_native_sdk/exported_symbols.json",
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

/// `clang` flags that mirror what `pebble build` passes per platform.
/// Reads the per-platform `defines` slice from `pebble-types`'s config table
/// and prepends `-D`. The platform-name, color/shape, capability, display
/// dimension, and `PBL_SDK_FROZEN` flags are all sourced from there.
/// Universal flags (`PBL_SDK_3`, the ARM ABI flags) are added at the call site.
fn clang_args_for(platform: Platform) -> Vec<String> {
	platform
		.config()
		.defines
		.iter()
		.map(|d| format!("-D{d}"))
		.collect()
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
