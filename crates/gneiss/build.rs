use pebble_build::{emit_platform_cfgs, platform_features, select_platform};

fn main() {
	for (env, _) in platform_features() {
		println!("cargo:rerun-if-env-changed={env}");
	}
	println!("cargo:rerun-if-env-changed=PBL_PLATFORM");
	emit_platform_cfgs(select_platform());
}
