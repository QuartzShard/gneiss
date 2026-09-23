# Resource kinds

A gneiss app that declares every currently-supported resource type (`Cargo.toml`)
and loads each one at runtime (`src/main.rs`). Build with
`cargo pebble build emery flint`.

## Per-platform files

A build looks for `<stem>~<platform>.<ext>` next to the declared `file` and only
falls back to `file` itself, so one declaration covers every watch and you add a
variant just for the ones that need their own asset. This example targets emery
(200x228, colour) and flint (144x168, black and white) and shows all three cases:

- `IMAGE_PNG` has a variant for both: `image~emery.png` and `image~flint.png`,
  each drawn for that screen's size and colour depth. `image.png` is what a
  third platform would get.
- `DATA_RAW` has a variant for one: flint reads `data~flint.bin`, emery falls
  back to `data.bin`. The app draws whichever it got as its title, so the two
  screenshots differ.
- `IMAGE_BITMAP` has none: every platform compiles the one `logo.png` for
  itself.

A resource whose declared `file` is missing and has no variant for the platform
being built is a build error naming the path it looked for.

## Prebaked samples

`logo~emery.pbi`, `logo~flint.pbi` and `font.pbf` are prebaked samples for the
`pbi` / `pbf` kinds. Those types copy an already-compiled file rather than
building one, so they were pulled straight out of each platform's resource pack
(the `.pbi`s are what `IMAGE_BITMAP` compiled to on that watch). A `.pbi` is
quantised for one platform's palette, which is why there is no shared `logo.pbi`
to fall back to. Everything else compiles from its source asset.
