# Message keys

Declares AppMessage keys in `Cargo.toml` and uses the generated
`message_keys::<name>` consts at the AppMessage boundary (`src/main.rs`).
Build with `cargo pebble build emery`.

Keys are declared with the upstream name DSL: a bare `"Name"` is one key,
`"Name[N]"` reserves `N` consecutive keys addressed as `message_keys::Name + i`.
Every declared name becomes a `u32` const; the build also writes the resolved
`name -> u32` map to `appKeys`/`messageKeys` in the bundle's `appinfo.json` and
to a `message_keys.json` next to the `.pbw` for a future companion to read.

Nothing round-trips under QEMU (no phone), so the outbox send just exercises
the write path; watch the logs for the resolved key values.
