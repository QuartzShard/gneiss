# A Gneiss Rust SDK for pebble watches

Gneiss (pronounced "nice") is a collection of crates that aim to provide a "pure rust" path to writing watchfaces and applications for the pebble watches.

| crate | provides | state |
|-------|----------|-------|
| gneiss | Safe API over the pebble SDK. | Entry/panic/alloc, logging, event services, AppMessage; UI/drawing unstarted |
| gneiss-macros | Macros for gneiss | `#[entry]`, `#[service]` |
| cargo-pebble | Project templating and Build tool for producing loadable `.pbw`, like `pebble-tool` for rust | MVP |
| pebble-sys | FFI surface for calling pebbleOS "syscalls" without linking `pebble.a`, generated from the upstream SDK by the `xtask` | Done |
| pebble-types | Shared definitions of the pebble platforms, etc. `no-std` for use in pebble-sys | Done |
| pebble-build | Build-time utilities | MVP |

This project is early stage, but the build tooling and `pebble-sys` FFI path already produce real apps that load and run on real hardware, and the safe API is starting to shape up. UI, typed resource handling still in the works.

 If this project interests you, do feel free to get in touch with me, quartzshard@quartzshard.com via email or @quartzshard:ethicallysourced.yachts on matrix. I'm unlikely to accept unsolicited PRs for the early stage stuff pre-gneiss while I get an MVP together.

## Usage
Clone the repo and build `cargo-pebble`, then stick it somewhere on your path (or add the build dir to your path) to get `cargo pebble` to work.

`cargo pebble new <name> -p emery,<other platforms> /path/to/gneiss/crates/gneiss/ [watchface|application]` to generate a template app, which you can build and run as is. If you want a raw `pebble-sys` app with no `gneiss`, crib from `examples/sys-only/`. See `crates/pebble-types/lib.rs` if you're not sure which platform is which watch.

`cargo pebble build emery <other platforms>` to build your application bundle, which you can use the usual `pebble install --phone <ip> ./target/thumbv7m-pebble/release/<name>.pbw` to sideload on to your watch (after enabling dev connection in the app), or `pebble install --emulator emery --logs ./target/thumbv7m-pebble/release/<name>.pbw` to run on emulator.

`gneiss` passes through watchface targets to `pebble_sys`, which exports per-watch bindings (re-exported as `gneiss::sys`) to the upstream API. The `alloc` feature also includes a `GlobalAlloc` over the PebbleOS `malloc`/`free`/`realloc`, unlocking `Vec` and friends, though adds about 5K to the binary. It's on by default, your know if you don't want it. I'd recommend local buffers and arenas where appropriate to make sure you're minimising heap fragmentation/usage - OOMs panic!

The `gneiss::entry` will register your services with the watch, call your `main()` and then hand over to the event loop.

You can annotate free-standing functions with `gneiss::service(SomeService)` to have the OS call your handler on that event. Some will take arguments to control what kinds of event cause a callback, and some take Optional context pointers. See the `service` module for details.

Resources must be declared in Cargo.toml, an example is provided in the template. These can currently be of type:
- raw (no processing performed, copied as-is), 
- bitmap (Any `image` supported image -> .pbi bitmap image),
- png (Any `image` supported image -> indexed .png image),
- pbi (already existing pebble bitmap, copied raw),
- font (freetype -> .pbf) / pbf (already existing, copied raw),
- icon (app menu icon, a special-case bitmap wired into the app header),
- vibe -> Haptics.
- pdc -> Pebble Draw Command (vector format)
Coming soon™:
- mo -> i18n

A resource can ship a different file per watch: name a sibling of the declared `file` `<stem>~<tag>.<ext>` and it is used on the platforms carrying that tag. The tags are upstream's — the platform name, `color`/`bw`, `rect`/`round`, dimensions like `200w`, and capabilities like `touch` — and a file may carry several (`image~color~rect.png`). A sibling naming the platform wins, then the one matching the most tags, then the untagged `file`. `examples/resources` demos it.

Happy programming!

## License
MPL-2.0 (see LICENSE), except the vendored headers from the pebble SDK/PebbleOS, which are Apache-2.0, see NOTICE.

## On the use of LLMs
This is not a "vibe coding" project and low quality slop is not wanted or welcome. That said - use of LLM tools as research aids, documentation drafting, commit message authoring, mechanical refactors/plumbing/etc, is Fine By Me. Just be sensible in scope and application, like with any other tool. **Never commit code you don't understand.**
