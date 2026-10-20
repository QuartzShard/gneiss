# A Gneiss Rust SDK for pebble watches

Gneiss (pronounced "nice") is a collection of crates that aim to provide a "pure rust" path to writing watchfaces and applications for the pebble watches. It's standalone, you don't need the C SDK or `pebble-tool` on your machine to build `gneiss` apps and install them on your watch.

| crate | provides |
|-------|----------|
| gneiss | The safe SDK: windows and layers, services, AppMessage, resources, logging. |
| gneiss-macros | `#[gneiss::entry]` and `#[gneiss::service]` |
| cargo-gneiss | Project templating, build tool for producing loadable `.pbw`, and installer, like `pebble-tool` for rust |
| gneiss-sys | FFI surface for calling pebbleOS "syscalls" without linking `pebble.a`, generated from the upstream SDK by the `xtask` |
| gneiss-types | Shared definitions of the pebble platforms, etc. `no-std` for use in gneiss-sys |
| gneiss-build | Build-time resource compiling and bundling, used from your app's `build.rs` |


### Tools/Setup
`cargo install cargo-gneiss` (or `cargo install --path crates/cargo-gneiss` from a checkout) gets you `cargo gneiss`. It builds on stable; the apps it generates pin a nightly in their `rust-toolchain.toml`, which rustup fetches on the first build.

`cargo gneiss new <name> -p emery,<other platforms> [watchface|application]` to generate a template app, which you can build and run as is. See `crates/gneiss-types/src/lib.rs` if you're not sure which platform is which watch. The template depends on the gneiss version matching your `cargo-gneiss`; to build against a local checkout or fork instead, add to its Cargo.toml:

```toml
[patch.crates-io]
gneiss = { path = "/path/to/gneiss/crates/gneiss" }
gneiss-build = { path = "/path/to/gneiss/crates/gneiss-build" }
```

`cargo gneiss build <optional platform specifiers>` to build your application bundle.

`cargo gneiss install --phone <ip> --logs` to sideload it on to your watch through the phone app (enable the developer connection in the app first), with `--logs` printing your app's logs until Ctrl-C. `cargo gneiss install --emulator <watch>` installs to an emulator that `pebble-tool` is already running, e.g. one started with `pebble install --emulator <watch>`. 

`gneiss` passes through watchface features to `gneiss_sys`, which exports per-watch bindings (re-exported as `gneiss::sys`) to the upstream API. To declare your app supports a watch, add an `emery = ["gneiss/emery"]` feature-forward to your Cargo.toml. `cargo-gneiss` looks for these during build. You can enable your own features with these, so long as one of the things enabled is the gneiss feature.

### Alloc
 `gneiss::entry` registers `gneiss::alloc::PebbleAlloc`, a `GlobalAlloc` over the PebbleOS `malloc`/`free`/`realloc`. You can supply `#[gneiss::entry(global_alloc = your::Allocator)]` to use your own. OOMs panic!
Internally, the `Allocator` interface is used to specify that internal heap allocations via the syscall interface are on the `PebbleAlloc` heap, regardless of your global allocator. I'd strongly recommend reaching for small buffers, pre/static allocation, bump allocators/arenas, and so on within your application to keep a lid on heap usage and fragmentation, _especially_ on old watches.

### Services
The `gneiss::entry` will call your `main()`, register your services with the watch, and then hand over to the event loop.
You can annotate free-standing functions with `gneiss::service(SomeService)` to have the OS call your handler on that event. Some will take arguments to control what kinds of event cause a callback, and some take optional context pointers. See the `service` module for details.

### AppMessage
Name your message keys in Cargo.toml (`message_keys = ["Command", "Samples[8]"]` under `[package.metadata.pebble]`) and they're generated as typed keys in a `message_keys` module, and listed in the bundle for your phone-side companion. Receive with `AppMessageService`, send with `link::Outbox`. `examples/message_keys` shows both directions.

### UI
A `Window<C>` loads a `C: Layout` when it's pushed and drops it when it leaves the stack, and keeps optional persistent data, the layout's `Data`, for its whole life. Layouts own layers and shared resources, Layers own what they show: `TextLayer`, `BitmapLayer`, and `DataLayer<T>` for your own state drawn by your own function.
`ui::graphics` provides `GContext` drawing functionality, and implements an `embedded-graphics` compatible framebuffer over it. 
Navigation is queued on a `UiQueue` and runs once the current handler returns. Anything that could otherwise alias a window's state takes a `UiGuard` token: give your `main` a `ui: &mut UiGuard` argument to set up and push your first window, since an app with no window on the stack exits.
`examples/resources` builds a whole screen this way.

### Resources
Resources must be declared in Cargo.toml; the `gneiss::resource` docs end with a declaration for every type. These can currently be of type:
- raw (no processing performed, copied as-is), 
- bitmap (Any `image` supported image -> .pbi bitmap image),
- png (Any `image` supported image -> indexed .png image),
- pbi (already existing pebble bitmap, copied raw),
- font (freetype -> .pbf) / pbf (already existing, copied raw),
- icon (app menu icon, a special-case bitmap wired into the app header),
- vibe -> Haptics.
- pdc -> Pebble Draw Command (vector format, image or animation)
- svg -> Pebble Draw Command, compiled from an svg file (or a directory of them for an animation)

Coming soon™:
- mo -> i18n

A resource can ship a different file per watch: name a sibling of the declared `file` `<stem>~<tag>.<ext>` and it is used on the platforms carrying that tag. The tags are upstream's: the platform name, `color`/`bw`, `rect`/`round`, dimensions like `200w`, and capabilities like `touch`. A file may carry several (`image~color~rect.png`). A sibling naming the platform wins, then the one matching the most tags, then the untagged `file`. `examples/resources` demos it.

### Logging, panics and size
`gneiss::log`'s macros write to the app log, which `cargo gneiss install --logs` prints. The `debug-logs` feature (on by default) keeps `debug!`/`verbose!`; turn it off to strip them from release builds. With only `logs`, `info!`/`warn!`/`error!` stay; without either, no logs compile in at all, though panics still log.
Apps build with `-Cpanic=immediate-abort`, which leaves the panic and `core::fmt` machinery out of the binary: a panic closes the app and prints no diagnostics. Format text with `gneiss::pbl_fmt_write!`, which formats through the firmware's `snprintf`. An app that wants a formatting panic handler can enable gneiss's `user-panic-handler` feature and bring its own at the cost of 1-2KiB of `core::fmt`. For deliberate panics/expect, use `pbl_panic!`/`.pbl_expect()` to log via the system `snprintf` formatting before the panic.

## Thanks for reading
If this project interests you, do feel free to get in touch with me, quartzshard@quartzshard.com via email or @quartzshard:ethicallysourced.yachts on matrix. I'm unlikely to accept unsolicited PRs for the early stage stuff pre-gneiss while I get an MVP together.
Happy programming!

## License
MPL-2.0 (see LICENSE), except the vendored headers from the pebble SDK/PebbleOS, which are Apache-2.0, see NOTICE.

## On the use of LLMs
This is not a "vibe coding" project and low quality slop is not wanted or welcome. That said - use of LLM tools as research aids, documentation drafting, commit message authoring, mechanical refactors/plumbing/etc, is Fine By Me. Just be sensible in scope and application, like with any other tool. **Never commit code you don't understand.**
