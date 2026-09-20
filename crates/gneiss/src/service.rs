//! ISR-Like services for watchapps to subscribe to with the #[gneiss::service] macro
use crate::{accel, compass, connection, focus, sys, time};

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
/// #[gneiss::service(TickService, time::TimeUnits::HOURS)]
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_TickService")]
			static GUARD: u8 = 0;
		};
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
	type Handler = fn(sys::BatteryChargeState);
	type RawHandler = unsafe extern "C" fn(sys::BatteryChargeState);
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_BatteryService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(charge: sys::BatteryChargeState) {
			$f(charge)
		}
	};
}

/// Called when backlight enables/disables
pub struct BacklightService;
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_BacklightService")]
			static GUARD: u8 = 0;
		};
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_AccelTapService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(axis: sys::AccelAxisType, direction: i32) {
			$f(axis.into(), direction.into())
		}
	};
}

/// Called with a batch of processed accelerometer samples, every `SubscribeArg` samples.
/// Shares the OS subscription slot with [`AccelRawDataService`].
pub struct AccelDataService;
impl Sealed for AccelDataService {}
impl Service for AccelDataService {
	type Handler = fn(&[accel::AccelData]);
	type RawHandler = unsafe extern "C" fn(*mut sys::AccelData, u32);
	type SubscribeArg = u32;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering AccelDataService handler;");
		unsafe { sys::accel_data_service_subscribe(a, Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelDataService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_AccelData")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(data: *mut sys::AccelData, num_samples: u32) {
			$f(if data.is_null() {
				&[]
			} else {
				::core::slice::from_raw_parts(data.cast(), num_samples as usize)
			})
		}
	};
}

/// Called with a batch of raw accelerometer samples and the timestamp (in ms) of the
/// first sample, every `SubscribeArg` samples.
/// Shares the OS subscription slot with [`AccelDataService`].
pub struct AccelRawDataService;
impl Sealed for AccelRawDataService {}
impl Service for AccelRawDataService {
	type Handler = fn(&[accel::Sample], u64);
	type RawHandler = unsafe extern "C" fn(*mut sys::AccelRawData, u32, u64);
	type SubscribeArg = u32;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering AccelRawDataService handler;");
		unsafe { sys::accel_raw_data_service_subscribe(a, Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelRawDataService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_AccelData")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(data: *mut sys::AccelRawData, num_samples: u32, timestamp: u64) {
			$f(
				if data.is_null() {
					&[]
				} else {
					::core::slice::from_raw_parts(data.cast(), num_samples as usize)
				},
				timestamp,
			)
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_CompassService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(headings: sys::CompassHeadingData) {
			$f(headings.into())
		}
	};
}

/// Called on app focus gain/loss
pub struct AppFocusService;
impl Sealed for AppFocusService {}
impl Service for AppFocusService {
	type Handler = fn(focus::Event);
	type RawHandler = fn() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool));
	type SubscribeArg = focus::SubscribeTo;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering AppFocusService handler;");
		let handlers = f();
		let handlers = sys::AppFocusHandlers {
			will_focus: a.contains(focus::SubscribeTo::WILL).then_some(handlers.0),
			did_focus:  a.contains(focus::SubscribeTo::DID).then_some(handlers.1),
		};
		unsafe { sys::app_focus_service_subscribe_handlers(handlers) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AppFocusService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_AppFocusService")]
			static GUARD: u8 = 0;
		};
		fn $r() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool)) {
			unsafe extern "C" fn __gneiss_app_focus_will(gaining: bool) {
				if gaining {
					$f(::gneiss::focus::Event::Gaining)
				} else {
					$f(::gneiss::focus::Event::Losing)
				}
			}
			unsafe extern "C" fn __gneiss_app_focus_did(gained: bool) {
				if gained {
					$f(::gneiss::focus::Event::Gained)
				} else {
					$f(::gneiss::focus::Event::Lost)
				}
			}
			(__gneiss_app_focus_will, __gneiss_app_focus_did)
		}
	};
}

/// Called on connection state changes for the phone app and/or `PebbleKit` companion
pub struct ConnectionService;
impl Sealed for ConnectionService {}
impl Service for ConnectionService {
	type Handler = fn(connection::Event);
	type RawHandler = fn() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool));
	type SubscribeArg = connection::SubscribeTo;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering ConnectionService handler;");
		let handlers = f();
		let handlers = sys::ConnectionHandlers {
			pebble_app_connection_handler: a
				.contains(connection::SubscribeTo::APP)
				.then_some(handlers.0),
			pebblekit_connection_handler:  a
				.contains(connection::SubscribeTo::PEBBLEKIT)
				.then_some(handlers.1),
		};
		unsafe { sys::connection_service_subscribe(handlers) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! ConnectionService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_ConnectionService")]
			static GUARD: u8 = 0;
		};
		fn $r() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool)) {
			unsafe extern "C" fn __gneiss_connection_app(connected: bool) {
				if connected {
					$f(::gneiss::connection::Event::AppConnected)
				} else {
					$f(::gneiss::connection::Event::AppDisconnected)
				}
			}
			unsafe extern "C" fn __gneiss_connection_pebblekit(connected: bool) {
				if connected {
					$f(::gneiss::connection::Event::PebbleKitConnected)
				} else {
					$f(::gneiss::connection::Event::PebbleKitDisconnected)
				}
			}
			(__gneiss_connection_app, __gneiss_connection_pebblekit)
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
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_WakeupService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(id: sys::WakeupId, reason: i32) {
			$f(id, reason)
		}
	};
}

/// Private trait bound means external modules cannot implement Service, but can still see it.
mod seal {
	pub trait Sealed {}
}
use seal::Sealed;
