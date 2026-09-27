//! ISR-Like services for watchapps to subscribe to with the #[gneiss::service] macro
use core::{marker::PhantomData, mem::MaybeUninit, ptr};

#[cfg(pbl_touch)]
use crate::touch;
use crate::{accel, compass, connection, focus, health, link, sys, time, wakeup};

pub struct NoneT;

/// Outlines an ISR-like Service fn registered to be called by the firmware.
pub trait Service: Sealed {
	type Handler: Copy;
	type RawHandler: Copy;
	type SubscribeArg;
	type Context;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg);
}

pub trait Peekable: Service {
	type PeekArgs;
	type Output;

	/// Peeks the service, returning its last reading as converted types
	fn peek(arg: Self::PeekArgs) -> Self::Output;
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
	type Context = NoneT;
	type Handler = fn(time::TickTime, time::TimeUnits);
	type RawHandler = unsafe extern "C" fn(*mut sys::tm, sys::TimeUnits);
	type SubscribeArg = time::TimeUnits;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering TickService handler;");
		unsafe { sys::tick_timer_service_subscribe(a.bits(), Some(f)) }
	}
}
impl Peekable for TickService {
	type Output = time::TickTime;
	type PeekArgs = ();

	fn peek(arg: Self::PeekArgs) -> Self::Output {
		// SAFETY: Time returns i32, &i32 is always valid, localtime has no documented fail state
		unsafe { *sys::localtime(&sys::time(ptr::null_mut())) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! TickService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_TickService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(tm: *mut sys::tm, unit: sys::TimeUnits) {
			// SAFETY: *mut tm is valid for the duration of the callback, after which it is
			// cleaned up - reading now is valid and correct, passing as a ref could escape.
			$f(
				unsafe { tm.read() },
				::gneiss::time::TimeUnits::from_bits_retain(unit),
			)
		}
	};
}

/// Called on battery state changes
pub struct BatteryService;
impl Sealed for BatteryService {}
impl Service for BatteryService {
	type Context = NoneT;
	type Handler = fn(sys::BatteryChargeState);
	type RawHandler = unsafe extern "C" fn(sys::BatteryChargeState);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::debug!("Registering BatteryService handler;");
		unsafe { sys::battery_state_service_subscribe(Some(f)) }
	}
}
impl Peekable for BatteryService {
	type Output = sys::BatteryChargeState;
	type PeekArgs = ();

	/// Query the state of the battery
	fn peek(_arg: Self::PeekArgs) -> Self::Output {
		unsafe { sys::battery_state_service_peek() }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! BatteryService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_BatteryService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(charge: sys::BatteryChargeState) {
			$f(charge)
		}
	};
}

/// Called when backlight enables/disables.
///
/// Absent on the frozen-SDK watches: their `pebble.h` stubs
/// `backlight_service_subscribe` out to a no-op macro. Upstream's only
/// backlight define, `PBL_RGB_BACKLIGHT`, marks a colour-capable backlight
/// (emery) rather than the presence of this service.
#[cfg(not(pbl_sdk_frozen))]
pub struct BacklightService;
#[cfg(not(pbl_sdk_frozen))]
impl Sealed for BacklightService {}
#[cfg(not(pbl_sdk_frozen))]
impl Service for BacklightService {
	type Context = NoneT;
	type Handler = fn(bool);
	type RawHandler = unsafe extern "C" fn(bool);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::debug!("Registering BacklightService handler;");
		unsafe { sys::backlight_service_subscribe(Some(f)) }
	}
}
#[cfg(not(pbl_sdk_frozen))]
#[macro_export]
#[doc(hidden)]
macro_rules! BacklightService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(accel::Axis, accel::Direction);
	type RawHandler = unsafe extern "C" fn(sys::AccelAxisType, core::ffi::c_int);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::debug!("Registering AccelTapService handler;");
		unsafe { sys::accel_tap_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelTapService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(&[accel::AccelData]);
	type RawHandler = unsafe extern "C" fn(*mut sys::AccelData, u32);
	type SubscribeArg = core::num::NonZeroU32;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering AccelDataService handler;");
		unsafe { sys::accel_data_service_subscribe(a.get(), Some(f)) }
	}
}
impl Peekable for AccelDataService {
	type Output = Option<accel::AccelData>;
	type PeekArgs = ();

	/// Peek one accel sample if available.
	/// Older watches may return Some(garbage) if accel is not running, as they never flag an error case
	fn peek(_arg: Self::PeekArgs) -> Self::Output {
		let mut dat = MaybeUninit::uninit();
		let ret = unsafe { sys::accel_service_peek(dat.as_mut_ptr()) };
		// SAFETY: 0 == OK, dat is populated
		(ret == 0).then(|| unsafe { accel::AccelData::from(dat.assume_init()) })
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelDataService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(&[accel::Sample], u64);
	type RawHandler = unsafe extern "C" fn(*mut sys::AccelRawData, u32, u64);
	type SubscribeArg = core::num::NonZeroU32;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering AccelRawDataService handler;");
		unsafe { sys::accel_raw_data_service_subscribe(a.get(), Some(f)) }
	}
}
impl Peekable for AccelRawDataService {
	type Output = Option<(accel::Sample, u64)>;
	type PeekArgs = ();

	/// Peek one accel sample if available.
	/// Older watches may return Some(garbage) if accel is not running, as they never flag an error case
	fn peek(_arg: Self::PeekArgs) -> Self::Output {
		let mut dat = MaybeUninit::uninit();
		let ret = unsafe { sys::accel_service_peek(dat.as_mut_ptr()) };
		// SAFETY: 0 == OK, dat is populated
		(ret == 0)
			.then(|| unsafe { accel::AccelData::from(dat.assume_init()) })
			.map(|d| (d.sample(), d.timestamp()))
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AccelRawDataService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(compass::HeadingData);
	type RawHandler = unsafe extern "C" fn(sys::CompassHeadingData);
	type SubscribeArg = compass::DeltaAngle;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering CompassService handler;");
		unsafe {
			sys::compass_service_set_heading_filter(a.inner().abs());
			sys::compass_service_subscribe(Some(f));
		}
	}
}
impl Peekable for CompassService {
	type Output = compass::HeadingData;
	type PeekArgs = ();

	/// Peek the compass service
	fn peek(_arg: Self::PeekArgs) -> Self::Output {
		let mut data = MaybeUninit::uninit();
		if unsafe { sys::compass_service_peek(data.as_mut_ptr()) } != 0 {
			panic!("Expected infallible, please report bug uptstream");
		};
		// SAFETY: peek deocumented to be infallible, we panic if it lied
		compass::HeadingData::from(unsafe { data.assume_init() })
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! CompassService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(focus::Event);
	type RawHandler = fn() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool));
	type SubscribeArg = focus::SubscribeTo;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering AppFocusService handler;");
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
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(connection::Event);
	type RawHandler = fn() -> (unsafe extern "C" fn(bool), unsafe extern "C" fn(bool));
	type SubscribeArg = connection::SubscribeTo;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::debug!("Registering ConnectionService handler;");
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
impl Peekable for ConnectionService {
	type Output = connection::State;
	type PeekArgs = ();

	/// Return the current [`connection::State`] of the watch
	fn peek(_arg: Self::PeekArgs) -> Self::Output {
		unsafe {
			connection::State {
				app:       sys::connection_service_peek_pebble_app_connection(),
				pebblekit: sys::connection_service_peek_pebblekit_connection(),
			}
		}
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! ConnectionService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
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
	type Context = NoneT;
	type Handler = fn(wakeup::WakeupId, wakeup::Cookie);
	type RawHandler = unsafe extern "C" fn(sys::WakeupId, i32);
	type SubscribeArg = ();

	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		crate::log::debug!("Registering WakeupService handler;");
		unsafe { sys::wakeup_service_subscribe(Some(f)) }
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! WakeupService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_WakeupService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(id: sys::WakeupId, reason: i32) {
			$f(
				::gneiss::wakeup::WakeupId(id),
				::gneiss::wakeup::Cookie(reason),
			)
		}

		fn __gneiss_wakeup_launch() {
			unsafe {
				let (mut id, mut cookie) = (0, 0);
				// False when not a wakeup, writes (id, cookie) when true.
				if ::gneiss::sys::wakeup_get_launch_event(&mut id, &mut cookie) {
					$f(
						::gneiss::wakeup::WakeupId(id),
						::gneiss::wakeup::Cookie(cookie),
					)
				}
			}
		}

		#[::gneiss::linkme::distributed_slice(GNEISS_LAUNCH_HOOKS)]
		#[linkme(crate = ::gneiss::linkme)]
		static WAKEUP_LAUNCH_HOOK: fn() = __gneiss_wakeup_launch;
	};
}

/// Requires alloc for the context storage. Ommiting or passing None to the SubscribeArg will skip
/// context alloc
///
/// Only on watches with a touch panel (`PBL_TOUCH`).
#[cfg(pbl_touch)]
pub struct TouchService<T = NoneT>(PhantomData<T>);
#[cfg(pbl_touch)]
impl<T> Sealed for TouchService<T> {}
#[cfg(pbl_touch)]
impl<T> Service for TouchService<T> {
	type Context = T;
	type Handler = for<'a> fn(touch::Event, Option<&'a mut T>);
	type RawHandler = unsafe extern "C" fn(*const sys::TouchEvent, *mut core::ffi::c_void);
	type SubscribeArg = Option<T>;

	#[cfg(feature = "alloc")]
	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		extern crate alloc;
		use alloc::boxed::Box;
		// into_raw leaks, so the returned memory is valid forevermore
		let ctx: *mut Self::Context = a.map(Box::new).map_or(core::ptr::null_mut(), Box::into_raw);
		Self::__subscribe(f, ctx)
	}

	#[cfg(not(feature = "alloc"))]
	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		Self::__subscribe(f, core::ptr::null_mut())
	}
}
#[cfg(pbl_touch)]
impl<T> TouchService<T> {
	fn __subscribe(f: <Self as Service>::RawHandler, ctx: *mut T) {
		crate::log::debug!("Registering TouchService handler;");
		// SAFETY: It's always safe to cast to c_void.
		unsafe { sys::touch_service_subscribe(Some(f), ctx.cast()) }
	}
}
#[cfg(pbl_touch)]
#[macro_export]
#[doc(hidden)]
macro_rules! TouchService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_TouchService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(event: *const sys::TouchEvent, ctx: *mut core::ffi::c_void) {
			// SAFETY: *const event is valid for the duration of the callback, after which it is
			// cleaned up - reading now is valid and correct, passing as a ref could escape. We know
			// ctx is valid because *we* leaked it.
			$f(
				::gneiss::touch::Event::from(unsafe { event.read() }),
				ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
					.as_mut(),
			)
		}
	};
}

/// Requires alloc for the context storage. Ommiting or passing None to the SubscribeArg will skip
/// context alloc
pub struct HealthService<T>(PhantomData<T>);
impl<T> Sealed for HealthService<T> {}
impl<T> Service for HealthService<T> {
	type Context = T;
	type Handler = for<'a> fn(health::Event, Option<&'a mut T>);
	type RawHandler = unsafe extern "C" fn(u8, *mut core::ffi::c_void);
	type SubscribeArg = Option<T>;

	#[cfg(feature = "alloc")]
	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		extern crate alloc;
		use alloc::boxed::Box;
		// into_raw leaks, so the returned memory is valid forevermore
		let ctx: *mut Self::Context = a.map(Box::new).map_or(core::ptr::null_mut(), Box::into_raw);
		Self::__subscribe(f, ctx)
	}

	#[cfg(not(feature = "alloc"))]
	fn subscribe(f: Self::RawHandler, _a: Self::SubscribeArg) {
		Self::__subscribe(f, core::ptr::null_mut())
	}
}

impl<T> HealthService<T> {
	fn __subscribe(f: <Self as Service>::RawHandler, ctx: *mut T) {
		crate::log::debug!("Registering HealthService handler;");
		// SAFETY: It's always safe to cast to c_void.
		if !unsafe { sys::health_service_events_subscribe(Some(f), ctx.cast()) } {
			crate::log::error!("Health service subscribe failed, not enough heap.");
		}
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! HealthService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_HealthService")]
			static GUARD: u8 = 0;
		};
		unsafe extern "C" fn $r(event: u8, ctx: *mut core::ffi::c_void) {
			// SAFETY: We leaked ctx, we know it's valid
			$f(
				::gneiss::health::Event::from(event),
				ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
					.as_mut(),
			)
		}
	};
}

pub struct AppMessageHandlers {
	pub recieved: sys::AppMessageInboxReceived,
	pub dropped:  sys::AppMessageInboxDropped,
	pub sent:     sys::AppMessageOutboxSent,
	pub failed:   sys::AppMessageOutboxFailed,
}

/// Requires alloc for the context storage. Ommiting or passing None to the SubscribeArg will skip
/// context alloc
pub struct AppMessageService<T>(PhantomData<T>);
impl<T> Sealed for AppMessageService<T> {}
impl<T> Service for AppMessageService<T> {
	type Context = T;
	type Handler = for<'a> fn(link::AppMessageEvent<'a>, Option<&'a mut T>);
	type RawHandler = fn() -> AppMessageHandlers;
	type SubscribeArg = (link::SubscribeTo, link::BufferSizes, Option<T>);

	#[cfg(feature = "alloc")]
	fn subscribe(f: Self::RawHandler, mut a: Self::SubscribeArg) {
		extern crate alloc;
		use alloc::boxed::Box;
		// into_raw leaks, so the returned memory is valid forevermore
		let ctx: *mut Self::Context =
			a.2.take()
				.map(Box::new)
				.map_or(core::ptr::null_mut(), Box::into_raw);
		Self::__subscribe(f, a, ctx)
	}

	#[cfg(not(feature = "alloc"))]
	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		Self::__subscribe(f, a, core::ptr::null_mut())
	}
}
impl<T> AppMessageService<T> {
	fn __subscribe(
		f: <Self as Service>::RawHandler,
		a: <Self as Service>::SubscribeArg,
		ctx: *mut T,
	) {
		crate::log::debug!("Registering AppMessageService handler;");
		let mut handlers = f();
		// SAFETY: ctx is null or a pointer we leaked; the handlers are valid fn pointers.
		unsafe {
			sys::app_message_set_context(ctx.cast());
			sys::app_message_register_inbox_received(
				handlers
					.recieved
					.take_if(|_| a.0.contains(link::SubscribeTo::RECIEVED)),
			);
			sys::app_message_register_inbox_dropped(
				handlers
					.dropped
					.take_if(|_| a.0.contains(link::SubscribeTo::DROPPED)),
			);
			sys::app_message_register_outbox_sent(
				handlers
					.sent
					.take_if(|_| a.0.contains(link::SubscribeTo::SENT)),
			);
			sys::app_message_register_outbox_failed(
				handlers
					.failed
					.take_if(|_| a.0.contains(link::SubscribeTo::FAILED)),
			);
			let buf_sizes = a.1;
			let in_max = sys::app_message_inbox_size_maximum();
			let out_max = sys::app_message_outbox_size_maximum();

			let inbox_size = buf_sizes
				.inbox
				.unwrap_or(in_max)
				.max(sys::APP_MESSAGE_INBOX_SIZE_MINIMUM)
				.min(in_max);
			let outbox_size = buf_sizes
				.outbox
				.unwrap_or(out_max)
				.max(sys::APP_MESSAGE_OUTBOX_SIZE_MINIMUM)
				.min(out_max);

			#[allow(
				unused_unsafe,
				reason = "log macro emits its own `unsafe`, trips if called in an unsafe block"
			)]
			if sys::app_message_open(inbox_size, outbox_size) != sys::AppMessageResult_APP_MSG_OK {
				crate::log::error!("AppMessage open failed");
			}
		}
	}
}
#[macro_export]
#[doc(hidden)]
macro_rules! AppMessageService_raw {
	(handler: $f: ident, raw_handler: $r: ident, service: $s: ty) => {
		const _: () = {
			#[unsafe(export_name = "__gneiss_slot_AppMessageService")]
			static GUARD: u8 = 0;
		};
		fn $r() -> AppMessageHandlers {
			// SAFETY: *DictionaryIterator is INVALID past the end of this callback, pass a short
			// borrow to the gneiss handler, not the raw ptr
			unsafe extern "C" fn __gneiss_appmessage_recieve(
				dict: *mut sys::DictionaryIterator,
				ctx: *mut core::ffi::c_void,
			) {
				$f(
					::gneiss::link::AppMessageEvent::Recieved {
						inbox: unsafe { ::gneiss::dict::DictHandle::from_raw(dict) },
					},
					ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
						.as_mut(),
				)
			}
			unsafe extern "C" fn __gneiss_appmessage_dropped(
				reason: sys::AppMessageResult,
				ctx: *mut core::ffi::c_void,
			) {
				$f(
					::gneiss::link::AppMessageEvent::Dropped {
						reason: reason.into(),
					},
					ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
						.as_mut(),
				)
			}
			unsafe extern "C" fn __gneiss_appmessage_sent(
				dict: *mut sys::DictionaryIterator,
				ctx: *mut core::ffi::c_void,
			) {
				$f(
					::gneiss::link::AppMessageEvent::Sent {
						outbox: unsafe { ::gneiss::dict::DictHandle::from_raw(dict) },
					},
					ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
						.as_mut(),
				)
			}
			unsafe extern "C" fn __gneiss_appmessage_failed(
				dict: *mut sys::DictionaryIterator,
				reason: sys::AppMessageResult,
				ctx: *mut core::ffi::c_void,
			) {
				$f(
					::gneiss::link::AppMessageEvent::Failed {
						outbox: unsafe { ::gneiss::dict::DictHandle::from_raw(dict) },
						reason: reason.into(),
					},
					ctx.cast::<<$s as ::gneiss::service::Service>::Context>()
						.as_mut(),
				)
			}
			AppMessageHandlers {
				recieved: Some(__gneiss_appmessage_recieve),
				dropped:  Some(__gneiss_appmessage_dropped),
				sent:     Some(__gneiss_appmessage_sent),
				failed:   Some(__gneiss_appmessage_failed),
			}
		}
	};
}
/// Private trait bound means external modules cannot implement Service, but can still see it.
mod seal {
	pub trait Sealed {}
}
use seal::Sealed;
