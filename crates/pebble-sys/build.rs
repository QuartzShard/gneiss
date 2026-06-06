use std::{env::var, path::PathBuf};

use pebble_types::Platform;

const FEATURES: &[(&str, Platform)] = &[
    ("CARGO_FEATURE_APLITE", Platform::Aplite),
    ("CARGO_FEATURE_BASALT", Platform::Basalt),
    ("CARGO_FEATURE_CHALK", Platform::Chalk),
    ("CARGO_FEATURE_DIORITE", Platform::Diorite),
    ("CARGO_FEATURE_EMERY", Platform::Emery),
    ("CARGO_FEATURE_FLINT", Platform::Flint),
    ("CARGO_FEATURE_GABBRO", Platform::Gabbro),
];

/// Pick the target platform. Prefer mutually-exclusive Cargo features; fall
/// back to the `PBL_PLATFORM` env var so `cargo xtask` flows (and anything
/// else not driven through `cargo build --features <plat>`) keep working.
/// `mutually_exclusive_features` only enforces at lib compile time — build.rs
/// runs first, so do the check here too.
fn select_platform() -> Platform {
    let enabled: Vec<Platform> = FEATURES
        .iter()
        .filter_map(|(env, p)| var(env).ok().map(|_| *p))
        .collect();
    match enabled.as_slice() {
        [p] => *p,
        [] => var("PBL_PLATFORM")
            .expect(
                "no platform selected: enable exactly one of the `aplite`/`basalt`/`chalk`/\
                 `diorite`/`emery`/`flint`/`gabbro` features, or set PBL_PLATFORM",
            )
            .parse()
            .expect("invalid PBL_PLATFORM"),
        many => panic!(
            "multiple platforms enabled ({:?}); choose exactly one",
            many.iter().map(|p| p.as_str()).collect::<Vec<_>>()
        ),
    }
}

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
    // (emitted by cargo-pebble) finds it via `link-arg=-Tpebble_app.ld`.
    std::fs::write(out_dir.join("pebble_app.ld"), resolved)
        .expect("Failed to write resolved linker script");

    for (env, _) in FEATURES {
        println!("cargo:rerun-if-env-changed={env}");
    }
    println!("cargo:rerun-if-env-changed=PBL_PLATFORM");
    println!("cargo:rerun-if-changed=pebble_app_rs.ld.template");
    println!("cargo:rustc-link-search={}", out_dir.display());
}
