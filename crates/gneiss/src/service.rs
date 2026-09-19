//! ISR-Like services for watchapps to subscribe to with the #[gneiss::service] macro
use crate::{accel, compass, sys, time};

/// Outlines an ISR-like Service fn registered to be called by the firmware.
pub trait Service: Sealed {
	type Handler: Copy;
	type RawHandler: Copy;
	type SubscribeArg;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg);
}

/// Called every time the time elapses the `TimeUnit` used in the `SubscribeArg`
/// For instance, a once-per-hour routine:
/// ```ignore
/// #[gneiss::service(time::TimeUnits::HOURS)]
/// fn hourly(time::TickTime, time:::TimeUnits) {
///     ///...
/// }
/// ```
pub struct TickService;
impl Sealed for TickService {}
impl Service for TickService {
	type Handler = fn(time::TickTime, time::TimeUnits);
	type RawHandler = unsafe extern "C" fn(*mut sys::tm, sys::TimeUnits);
	type SubscribeArg = time::TimeUnits;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering TickService handler;");
		unsafe { sys::tick_timer_service_subscribe(a.bits(), Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! TickService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(tm: *mut sys::tm, unit: sys::TimeUnits) {
			$f(
				tm.read_volatile(),
				::gneiss::time::TimeUnits::from_bits_retain(unit),
			)
		}
	};
}

/// Called on battery state changes
pub struct BatteryService;
impl Sealed for BatteryService {}
impl Service for BatteryService {
	type Handler = fn(BatteryChargeState);
	type RawHandler = unsafe extern "C" fn(BatteryChargeState);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::verbose!("Registering BatteryService handler;");
		unsafe { sys::battery_state_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! BatteryService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(charge: sys::BatteryChargeState) {
			$f(charge)
		}
	};
}

/// Called when backlight enables/disables
impl Sealed for BacklightService {}
impl Service for BacklightService {
	type Handler = fn(bool);
	type RawHandler = unsafe extern "C" fn(bool);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::verbose!("Registering BacklightService handler;");
		unsafe { sys::backlight_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! BacklightService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(on: bool) {
			$f(on)
		}
	};
}

/// Called when the watch registers a "tap" with the accelerometer
pub struct AccelTapService;
impl Sealed for AccelTapService {}
impl Service for AccelTapService {
	type Handler = fn(accel::Axis, accel::Direction);
	type RawHandler = unsafe extern "C" fn(sys::AccelAxisType, core::ffi::c_int);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::verbose!("Registering AccelTapService handler;");
		unsafe { sys::accel_tap_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelTapService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(axis: sys::AccelAxisType, direction: i32) {
			$f(axis.into(), direction.into())
		}
	};
}

/// Called when the heading changes by more than the supplied `DeltaAngle`
pub struct CompassService;
impl Sealed for CompassService {}
impl Service for CompassService {
	type Handler = fn(compass::HeadingData);
	type RawHandler = unsafe extern "C" fn(sys::CompassHeadingData);
	type SubscribeArg = compass::DeltaAngle;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering CompassService handler;");
		unsafe {
			sys::compass_service_set_heading_filter(a.inner().abs());
			sys::compass_service_subscribe(Some(f));
		}
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! CompassService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(headings: sys::CompassHeadingData) {
			$f(headings.into())
		}
	};
}

/// Called on app focus gain/loss
pub struct AppFocusService;
impl Sealed for AppFocusService {}
impl Service for AppFocusService {
	type Handler = fn(bool);
	type RawHandler = unsafe extern "C" fn(bool);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::verbose!("Registering AppFocusService handler;");

		unsafe { sys::app_focus_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AppFocusService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(gaining: bool) {
			$f(gaining)
		}
	};
}

pub struct WakeupService;
impl Sealed for WakeupService {}
impl Service for WakeupService {
	type Handler = fn(sys::WakeupId, i32);
	type RawHandler = unsafe extern "C" fn(sys::WakeupId, i32);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering WakeupService handler;");
		unsafe { sys::wakeup_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! WakeupService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(id: sys::WakeupId, reason: i32) {
			$f(id, reason)
		}
	};
}

/// Private trait bound means external modules cannot implement Service, but can still see it.
mod seal {
	pub trait Sealed {}
}
use pebble_sys::BatteryChargeState;
use seal::Sealed;
