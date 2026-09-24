# Resource kinds

A gneiss app that declares every currently-supported resource type (`Cargo.toml`)
and loads each one at runtime (`src/main.rs`).

## Tagged files

A build looks for siblings of the declared `file` named `<stem>~<tag>.<ext>`,
and falls back to `file` itself. The tags are upstream's, one set per platform:
its name (`emery`), its colour depth (`color`/`bw`), shape (`rect`/`round`),
screen size (`200w`, `228h`) and capabilities (`touch`, `mic`, `speaker`, ...).
A file may carry several: `image~color~rect.png`.

Ranking, most specific first:

1. a sibling naming the platform, e.g. `image~emery.png`
2. otherwise the sibling matching the most of that platform's tags
3. otherwise the untagged `file`

A tag the platform does not carry rules the sibling out entirely, two equally
specific siblings are a build error, and so is a tag no platform declares (a
typo cannot silently fall back).

Build it with `cargo pebble build emery flint chalk basalt aplite`: between them
those five cover colour and black and white, round and rect, and both SDK
vintages. What each one resolves to:

| resource | emery | chalk | basalt | flint | aplite |
|---|---|---|---|---|---|
| `IMAGE_PNG` (`image.png`) | `~emery` | `~color~round` | `~color` | `image.png` | `image.png` |
| `DATA_RAW` (`data.bin`) | `data.bin` | `data.bin` | `data.bin` | `~flint` | `~bw` |
| `IMAGE_PBI` (`logo.pbi`) | `~color` | `~color` | `~color` | `~bw` | `~bw` |
| `IMAGE_BITMAP` (`logo.png`) | compiled per platform from the one file | | | | |

Each rule shows up at least once. Emery takes `image~emery.png` by name even
though `image~color.png` matches it too; chalk takes `image~color~round.png` on
two tags over `image~color.png`'s one; the black and white watches match no PNG
sibling and fall back. Flint matches both `data~flint.bin` and `data~bw.bin` and
takes the name, while aplite takes `data~bw.bin`. The app draws `DATA_RAW` as
its title, so the screenshots differ.

A resource whose declared `file` is missing and has no sibling for the platform
being built is a build error naming the resource, the platform and the path.

The app draws its title from `DATA_RAW`, the tagged PNG below it and the
prebaked `.pbi` below that, so a screenshot shows what each watch resolved to.
All five run in the emulator (`pebble install --emulator <platform>`).

## Prebaked samples

`logo~color.pbi`, `logo~bw.pbi` and `font.pbf` are prebaked samples for the
`pbi` / `pbf` kinds. Those types copy an already-compiled file rather than
building one, so they were pulled straight out of a resource pack (the `.pbi`s
are what `IMAGE_BITMAP` compiled to). A `.pbi` is quantised for a colour depth
rather than for a watch — the emery, basalt and chalk packs hold byte-identical
2-bit bitmaps, and flint and aplite byte-identical 1-bit ones — so two files
cover five platforms, and there is no shared `logo.pbi` to fall back to.
Everything else compiles from its source asset.
