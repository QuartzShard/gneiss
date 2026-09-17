use crate::{
	sys,
	time::{TickTime, TimeUnits},
};

/// Outlines an ISR-like Service fn registered to be called by the firmware.
pub trait Service: Sealed {
	type Handler: Copy;
	type RawHandler: Copy;
	type SubscribeArg;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg);
}

pub struct TickService;
#[macro_export]
#[doc(hidden)]
macro_rules! TickService_raw {
	(handler: $f: ident, raw_handler: $r: ident) => {
		unsafe extern "C" fn $r(tm: *mut sys::tm, unit: sys::TimeUnits) {
			$f(
				tm.read_volatile(),
				::gneiss::time::TimeUnits::from_bits_retain(unit as u32),
			)
		}
	};
}
impl TickService {
	pub unsafe extern "C" fn raw_handler() {}
}
impl Sealed for TickService {}
impl Service for TickService {
	type Handler = fn(time: TickTime, changed: TimeUnits);
	type RawHandler = unsafe extern "C" fn(*mut sys::tm, sys::TimeUnits);
	type SubscribeArg = TimeUnits;

	fn subscribe(f: Self::RawHandler, a: Self::SubscribeArg) {
		crate::log::verbose!("Registering TickHandler!");
		unsafe { sys::tick_timer_service_subscribe(a.bits(), Some(f)) }
	}
}
/// Private trait bound means external modules cannot implement Service, but can still see it.
mod seal {
	pub trait Sealed {}
}
use seal::Sealed;
