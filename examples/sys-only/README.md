# sys-only

Hello world over raw `gneiss-sys`, with no `gneiss` in the link
(`src/main.rs`). Build with `cargo gneiss build emery`.

Two reasons it exists:

- It is what `#[gneiss::entry]` generates, written out by hand: the panic
  handler, the exported `main` the loader jumps to, and the final
  `app_event_loop()` handoff. Service registration and the generated
  `resources`/`message_keys` modules are missing, since they need gneiss.
- It is the bisection layer for firmware updates. If this app still runs after
  an OTA and a gneiss app does not, the break is above the FFI surface, and
  vice versa.
