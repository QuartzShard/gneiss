# All services

A gneiss app that registers every `Service` and logs whatever each hands back
(`src/main.rs`). Companion to the `resources` example. Build with
`cargo gneiss build emery`.

The buttons subscribe and unsubscribe services at runtime: Select turns the
accelerometer and compass off and on, and Up and Down make the tick finer
(seconds) or coarser (hours). The top line shows the current settings.

Every distinct service slot is covered. `AccelRawDataService` is left out
because it shares `AccelDataService`'s subscription slot: registering both is a
deliberate link-time error, so swap one for the other to try raw samples.

Under QEMU most services stay quiet: no BLE peer (connection), no heart-rate
data (health), and a stubbed compass that reports `Unavailable` (the handler
gates on status and says so). That stub also calls the compass handler from
inside the subscribe, which Select does from its click handler, so the handler
takes the UI guard with `try_enter` and skips the update when it's held. Tick
and battery are the ones you will reliably see fire.
