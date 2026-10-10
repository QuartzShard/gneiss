//! Time, for [`TickService`](crate::service::TickService).

use crate::sys;
#[doc(inline)]
pub use crate::sys::tm as TickTime;

bitflags::bitflags! {
	/// Which units of the time changing should fire a tick.
	pub struct TimeUnits: u8 {
		const SECONDS   = sys::TimeUnits_SECOND_UNIT;
		const MINUTES   = sys::TimeUnits_MINUTE_UNIT;
		const HOURS     = sys::TimeUnits_HOUR_UNIT;
		const DAYS      = sys::TimeUnits_DAY_UNIT;
		const MONTHS    = sys::TimeUnits_MONTH_UNIT;
		const YEARS     = sys::TimeUnits_YEAR_UNIT;
	}
}
