# All services

A gneiss app that registers every `Service` and logs whatever each hands back
(`src/main.rs`). Companion to the `resources` example. Build with
`cargo gneiss build emery`.

Every distinct service slot is covered. `AccelRawDataService` is left out
because it shares `AccelDataService`'s subscription slot: registering both is a
deliberate link-time error, so swap one for the other to try raw samples.

Under QEMU most services stay quiet: no BLE peer (connection), no heart-rate
data (health), and a stubbed compass that reports `Unavailable` (the handler
gates on status and says so). Tick and battery are the ones you will reliably
see fire.
