# Stopwatch

A gneiss app timing to the millisecond (`src/main.rs`). Select starts and stops
it, Down resets it. Build with `cargo gneiss build emery`.

The firmware's clock measures the time, through `time::now()`. A
repeating `Timer`, held by the window's layout, only redraws the display while
the stopwatch runs: a late frame never skews the reading, and stopping it is
dropping the timer.

Under QEMU the emulator's clock can jump by up to a second between readings
(see `Timestamp`), so the display may flicker backwards. Watches don't.
