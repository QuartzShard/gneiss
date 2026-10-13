//! Time, for [`TickService`](crate::service::TickService).

use core::ops::{Add, AddAssign, Sub, SubAssign};

#[doc(inline)]
pub use crate::sys::tm as TickTime;
use crate::{service::TickService, sys};

bitflags::bitflags! {
	/// Which units of the time changing should fire a tick.
	#[derive(Clone, Copy)]
	pub struct TimeUnits: u8 {
		const SECONDS   = sys::TimeUnits_SECOND_UNIT;
		const MINUTES   = sys::TimeUnits_MINUTE_UNIT;
		const HOURS     = sys::TimeUnits_HOUR_UNIT;
		const DAYS      = sys::TimeUnits_DAY_UNIT;
		const MONTHS    = sys::TimeUnits_MONTH_UNIT;
		const YEARS     = sys::TimeUnits_YEAR_UNIT;
	}
}

/// The time to the millisecond, from `TickService::fine()`. Subtract two for the [`FineDuration`]
/// between them.
///
/// Wall-clock time: it jumps when the watch's clock is set, such as by the phone syncing it.
///
/// # Emulator
///
/// The emulator's clock takes the milliseconds from a counter that isn't aligned to the seconds, so
/// readings can jump by up to a second, either way, between calls. Watches read both from the clock.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FineTime {
	// Since the Unix epoch
	millis: u64,
}

impl FineTime {
	/// Milliseconds since the Unix epoch.
	pub fn as_millis(self) -> u64 {
		self.millis
	}

	/// Whole seconds since the Unix epoch.
	pub fn seconds(self) -> u64 {
		self.millis / 1000
	}
}

/// The span between two [`FineTime`]s, in milliseconds. Signed, as setting the watch's clock between
/// two readings can put the later one first. Saturates at about 24 days either way.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FineDuration {
	millis: i32,
}

impl FineDuration {
	pub const ZERO: Self = Self { millis: 0 };

	pub const fn from_millis(millis: i32) -> Self {
		Self { millis }
	}

	pub const fn as_millis(self) -> i32 {
		self.millis
	}
}

impl Sub for FineTime {
	type Output = FineDuration;

	/// The time from `earlier` to `self`.
	fn sub(self, earlier: Self) -> FineDuration {
		let millis = if self >= earlier {
			i32::try_from(self.millis - earlier.millis).unwrap_or(i32::MAX)
		} else {
			i32::try_from(earlier.millis - self.millis).map_or(i32::MIN, |m| -m)
		};
		FineDuration::from_millis(millis)
	}
}

impl FineTime {
	/// `millis` later, or earlier if negative.
	fn shifted(self, millis: i32) -> Self {
		let by = u64::from(millis.unsigned_abs());
		Self {
			millis: if millis < 0 {
				self.millis.saturating_sub(by)
			} else {
				self.millis.saturating_add(by)
			},
		}
	}
}

impl Add<FineDuration> for FineTime {
	type Output = Self;

	fn add(self, duration: FineDuration) -> Self {
		self.shifted(duration.millis)
	}
}

impl Sub<FineDuration> for FineTime {
	type Output = Self;

	fn sub(self, duration: FineDuration) -> Self {
		self.shifted(duration.millis.saturating_neg())
	}
}

impl AddAssign<FineDuration> for FineTime {
	fn add_assign(&mut self, duration: FineDuration) {
		*self = *self + duration;
	}
}

impl SubAssign<FineDuration> for FineTime {
	fn sub_assign(&mut self, duration: FineDuration) {
		*self = *self - duration;
	}
}

impl Add for FineDuration {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self::from_millis(self.millis.saturating_add(other.millis))
	}
}

impl Sub for FineDuration {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self::from_millis(self.millis.saturating_sub(other.millis))
	}
}

impl AddAssign for FineDuration {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl SubAssign for FineDuration {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

impl TickService {
	pub fn fine() -> FineTime {
		let mut seconds: i32 = 0;
		let mut millis: u16 = 0;
		unsafe { sys::time_ms(&raw mut seconds, &raw mut millis) };
		// it's not before 1970
		FineTime {
			millis: u64::try_from(seconds).unwrap_or(0) * 1000 + u64::from(millis),
		}
	}
}
