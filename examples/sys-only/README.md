# sys-only

Hello world over raw `pebble-sys`, with no `gneiss` in the link
(`src/main.rs`). Build with `cargo pebble build emery`.

Two reasons it exists:

- It is what `#[gneiss::entry]` generates, written out by hand: the panic
  handler, the `resources`/`message_keys` include modules, the exported `main`
  the loader jumps to, and the final `app_event_loop()` handoff. Only service
  registration is missing, since `#[gneiss::service]` needs gneiss.
- It is the bisection layer for firmware updates. If this app still runs after
  an OTA and a gneiss app does not, the break is above the FFI surface, and
  vice versa.

One constraint: declared message keys generate gneiss-typed consts, so a
raw-sys crate must leave `message_keys` out of `Cargo.toml`. The empty include
compiles fine.
