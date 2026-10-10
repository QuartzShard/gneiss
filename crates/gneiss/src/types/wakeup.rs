//! Token types for wakeup.

#[derive(Clone, Copy, PartialEq, Eq)]
/// The ID of the wakeup
pub struct WakeupId(pub i32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The "reason" supplied for wakeup. You can use this however you want, and the meaning probably
/// depends on the ID of the wakeup. An example:
/// ```no_run,standalone_crate
/// # use gneiss::{log, service::*, wakeup::*};
/// # gneiss::__doctest_slices!();
/// # struct AlarmReason;
/// # impl From<Cookie> for AlarmReason { fn from(_: Cookie) -> Self { AlarmReason } }
/// # fn handle_alarm_one(_: AlarmReason) {}
/// # fn increment_counter(_: i32) {}
/// #[gneiss::service(WakeupService)]
/// fn on_wake(id: WakeupId, cookie: Cookie) {
///     match (id, cookie) {
///         (WakeupId(1), cookie) => handle_alarm_one(AlarmReason::from(cookie)),
///         (WakeupId(2), cookie) => increment_counter(cookie.0),
///         (id, cookie) => unsafe { log::log_sys_fmt!(log::LogLevel::Warn, "Got unknown wakeup: Id(%d):Cookie(%d)",
///         id.0, cookie.0) }
///     }
/// }
/// ```
pub struct Cookie(pub i32);
