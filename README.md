# A Gneiss Rust SDK for pebble watches

Gneiss (pronounced "nice") is a collection of crates that aim to provide a "pure rust" path to writing watchfaces and applications for the pebble watches. It's standalone, you don't need to have the C SDK on your machine to build `gneiss` apps.

| crate | provides |
|-------|----------|
| gneiss | Safe API over the pebble SDK. |  
| gneiss-macros | Macros for gneiss |
| cargo-gneiss | Project templating and Build tool for producing loadable `.pbw`, like `pebble-tool` for rust | 
| pebble-sys | FFI surface for calling pebbleOS "syscalls" without linking `pebble.a`, generated from the upstream SDK by the `xtask` | 
| pebble-types | Shared definitions of the pebble platforms, etc. `no-std` for use in pebble-sys | 
| pebble-build | Build-time utilities | 


## Usage

### Tools/Setup
Clone the repo and build `cargo-gneiss`, then stick it somewhere on your path (or add the build dir to your path) to get `cargo gneiss` to work.

`cargo gneiss new <name> -p emery,<other platforms> /path/to/gneiss/crates/gneiss/ [watchface|application]` to generate a template app, which you can build and run as is. See `crates/pebble-types/lib.rs` if you're not sure which platform is which watch.

`cargo gneiss build <optional platform specifiers>` to build your application bundle, which you can use the usual `pebble install --phone <ip> ./target/thumbv7m-pebble/release/<name>.pbw` to sideload on to your watch (after enabling dev connection in the app), or `pebble install --emulator <watch> --logs ./target/thumbv7m-pebble/release/<name>.pbw` to run on emulator.

`gneiss` passes through watchface features to `pebble_sys`, which exports per-watch bindings (re-exported as `gneiss::sys`) to the upstream API. To declare your app supports a watch, add an `emery = ["gneiss/emery"]` feature-forward to your Cargo.toml. `cargo-gneiss` looks for these during build. You can enable your own features with these, so long as one of the things enabled is the gneiss feature.

### Alloc
 `gneiss::entry` registers `gneiss::alloc::PebbleAlloc`, a `GlobalAlloc` over the PebbleOS `malloc`/`free`/`realloc`. You can supply `#[gneiss::entry(global_alloc = your::Allocator)]` to use your own. OOMs panic!
Internally, the `Allocator` interface is used to specify that internal heap allocations via the syscall interface are on the `PebbleAlloc` heap, regardless of your global allocator. I'd strongly recommend reaching for small buffers, pre/static allocation, bump allocators/arenas, and so on within your application to keep a lid on heap usage and fragmentation, _especially_ on old watches.

### Services
The `gneiss::entry` will register your services with the watch, call your `main()` and then hand over to the event loop.
You can annotate free-standing functions with `gneiss::service(SomeService)` to have the OS call your handler on that event. Some will take arguments to control what kinds of event cause a callback, and some take optional context pointers. See the `service` module for details.

### UI
A `Window<C, D>` loads a `C: Layout` when it's pushed and drops it when it leaves the stack, and keeps optional persistent data `D` for its whole life. Layouts own layers and shared resources, Layers own what they show: `TextLayer`, `BitmapLayer`, and `DataLayer<T>` for your own state drawn by your own function.
`ui::graphics` provides `GContext` drawing functionality, and implements an `embedded-graphics` compatible framebuffer over it. 
Navigation is queued on a `Nav` and runs once the current handler returns. Anything that could otherwise alias a window's state takes a `UiGuard` token: give your `main` a `ui: &mut UiGuard` argument to set up and push your first window, since an app with no window on the stack exits.
`examples/resources` builds a whole screen this way.

### Resources
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

## Thanks for reading
If this project interests you, do feel free to get in touch with me, quartzshard@quartzshard.com via email or @quartzshard:ethicallysourced.yachts on matrix. I'm unlikely to accept unsolicited PRs for the early stage stuff pre-gneiss while I get an MVP together.
Happy programming!

## License
MPL-2.0 (see LICENSE), except the vendored headers from the pebble SDK/PebbleOS, which are Apache-2.0, see NOTICE.

## On the use of LLMs
This is not a "vibe coding" project and low quality slop is not wanted or welcome. That said - use of LLM tools as research aids, documentation drafting, commit message authoring, mechanical refactors/plumbing/etc, is Fine By Me. Just be sensible in scope and application, like with any other tool. **Never commit code you don't understand.**
