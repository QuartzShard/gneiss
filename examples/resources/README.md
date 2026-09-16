# Resource kinds

A gneiss app that declares every currently-supported resource type (`Cargo.toml`)
and loads each one at runtime (`src/main.rs`). Build with `cargo pebble build emery`.

`image.pbi` and `font.pbf` are pre-baked samples for the `pbi` / `pbf` kinds.
Those types copy an already-compiled file rather than building one, so they were
pulled straight out of a build's resource pack. Everything else compiles from
its source asset.
