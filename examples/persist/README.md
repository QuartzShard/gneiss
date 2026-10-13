# Persist

A lifetime counter: a gneiss app showing how long it has been open, over every
launch (`src/main.rs`). Build with `cargo gneiss build emery`.

The total lives in one persist key, declared with `persist_keys!`. It's
recovered at launch, counted up by the tick service each second, and saved when
the window unloads, from `Layout::unload`.
