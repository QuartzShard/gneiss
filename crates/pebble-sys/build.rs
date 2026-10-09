use std::{env::var, path::PathBuf};

use pebble_build::{emit_platform_cfgs, platform_features, select_platform};

fn main() {
	let in_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").expect("No in dir"));
	let out_dir = PathBuf::from(var("OUT_DIR").expect("No out dir"));

	let platform = select_platform();
	let cfg = platform.config();
	assert_eq!(
		var("TARGET").expect("No target"),
		platform.app_target_triple(),
		"pebble apps must build against the v7m ABI"
	);

	let template = std::fs::read_to_string(in_dir.join("pebble_app_rs.ld.template"))
		.expect("Cannot read linker template");
	let resolved = template.replace(
		"@MAX_APP_MEMORY_SIZE@",
		&format!("0x{:x}", cfg.max_app_memory_size),
	);
	// Use the conventional `pebble_app.ld` name so downstream `.cargo/config.toml`
	// (emitted by cargo-gneiss) finds it via `link-arg=-Tpebble_app.ld`.
	std::fs::write(out_dir.join("pebble_app.ld"), resolved)
		.expect("Failed to write resolved linker script");

	for (env, _) in platform_features() {
		println!("cargo:rerun-if-env-changed={env}");
	}
	emit_platform_cfgs(platform);
	println!("cargo:rerun-if-env-changed=PBL_PLATFORM");
	println!("cargo:rerun-if-changed=pebble_app_rs.ld.template");
	println!("cargo:rustc-link-search={}", out_dir.display());
}
