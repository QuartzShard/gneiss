# A Gneiss Rust SDK for pebble watches

Gneiss (pronounced "nice") is a collection of crates that aim to provide a "pure rust" path to writing watchfaces and applications for the pebble watches.

| crate | provides | state |
|-------|----------|-------|
| gneiss | Safe API over the pebble SDK. If you're making a watch and don't want "the C SDK but in rust", this is the crate for you | Unstarted |
| gneiss-macros | Macros for gneiss | Unstarted |
| cargo-pebble | Project templating and Build tool for producing loadable `.pbw`, like `pebble-tool` for rust | MVP |
| pebble-sys | FFI surface for calling pebbleOS "syscalls" without linking `pebble.a`, generated from the upstream SDK by the `xtask` | MVP |
| pebble-types | Shared definitions of the pebble platforms, etc. `no-std` for use in pebble-sys | MVP |

This project is very early stage, so don't expect to be running anything on your watch just yet. The roadmap is, broadly:
 - ~~`cargo-pebble` build producing a valid, loadable `.pbw` archive, proving that `pebble-sys` generates valid syscall shims and this can work at all.~~ DONE: You can now "Write it in Rust™"
 - `gneiss` + `gneiss-macros` providing basic provisions for binding event listeners, an entry point, appropriate panic handler, etc. This is where we become "useful" instead of "interesting"
 - `gneiss` providing well-designed, safe abstraction over the `pebble-sys` FFI so you can write compiler-guaranteed safe pebble watch-apps. Release Candidate here
 - `gneiss` allocator, maybe? Or just tell everyone to use static-backed arenas and `heapless`.

 If this project interests you, do feel free to get in touch with me, quartzshard@quartzshard.com via email or @quartzshard:ethicallysourced.yachts on matrix. I'm unlikely to accept unsolicited PRs for the early stage stuff pre-gneiss while I get an MVP together.

## Usage
Clone the repo and build `cargo-pebble`, then stick it somewhere on your path (or add the build dir to your path) to get `cargo pebble` to work.

`cargo pebble new <name> -p emery,<other platforms> /path/to/gneiss/crates/pebble-sys/ [watchface|application] --sys-only` to generate a template app, which you can build and run as is. See `crates/pebble-types/lib.rs` if you're not sure which platform is which watch.

`cargo pebble build emery <other platforms>` to build your application bundle, which you can use the usual `pebble install --phone <ip> ./target/thumb7m-pebble/release/<name>.pbw` to sideload on to your watch (after enabling dev connection in the app).

Happy programming!

## License
MPL-2.0 (see LICENSE), except the vendored headers from the pebble SDK/PebbleOS, which are Apache-2.0, see NOTICE.

## On the use of LLMs
This is not a "vibe coding" project and low quality slop is not wanted or welcome. That said - use of LLM tools as research aids, documentation drafting, commit message authoring, mechanical refactors/plumbing/etc, is Fine By Me. Just be sensible in scope and application, like with any other tool. **Never commit code you don't understand.**
