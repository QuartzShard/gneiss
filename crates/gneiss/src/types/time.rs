//! Time: the clock, for [`TickService`](crate::service::TickService), and the user's time
//! settings.

use core::{
	ffi::{CStr, c_char},
	ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{sys, ui::StackText};

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

/// A moment in time, to the millisecond, such as from [`now`]. Subtract two for the [`TimeDelta`]
/// between them.
///
/// Wall-clock time: it jumps when the watch's clock is set, such as by the phone syncing it.
///
/// # Emulator
///
/// The emulator's clock takes the milliseconds from a counter that isn't aligned to the seconds, so
/// readings can jump by up to a second, either way, between calls. Watches read both from the clock.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
	// Since the Unix epoch
	millis: u64,
}

impl Timestamp {
	/// `seconds` since the Unix epoch.
	pub const fn from_seconds(seconds: u64) -> Self {
		Self {
			millis: seconds * 1000,
		}
	}

	/// From a firmware timestamp: it's not before 1970.
	fn from_time_t(seconds: sys::time_t) -> Self {
		Self::from_seconds(u64::try_from(seconds).unwrap_or(0))
	}

	fn as_time_t(self) -> sys::time_t {
		sys::time_t::try_from(self.seconds()).unwrap_or(sys::time_t::MAX)
	}

	/// The date and time in the watch's time zone.
	pub fn local(self) -> DateTime {
		let t = self.as_time_t();
		// SAFETY: localtime fills the app's own tm and returns it; copied out before any other call
		DateTime(unsafe { *sys::localtime(&raw const t) })
	}

	/// The date and time in UTC.
	pub fn utc(self) -> DateTime {
		let t = self.as_time_t();
		// SAFETY: as for local, with gmtime's tm
		DateTime(unsafe { *sys::gmtime(&raw const t) })
	}

	/// As [`DateTime::format`], in the watch's time zone.
	///
	/// ```no_run
	/// # fn main() {
	/// let today = gneiss::time::now().format::<16>(c"%A %e");
	/// # }
	/// ```
	pub fn format<const N: usize>(self, format: &CStr) -> StackText<N> {
		self.local().format(format)
	}

	/// The moment it's this date and time in the watch's time zone. Values out of range roll over
	/// into the next field, such as day 32 into the next month.
	pub fn from_local(year: i32, month: Month, day: u8, hour: u8, minute: u8, second: u8) -> Self {
		// SAFETY: all-zero is a valid tm, of plain integers and bytes
		let mut tm: sys::tm = unsafe { core::mem::zeroed() };
		tm.tm_year = year - 1900;
		tm.tm_mon = month as i32 - 1;
		tm.tm_mday = day.into();
		tm.tm_hour = hour.into();
		tm.tm_min = minute.into();
		tm.tm_sec = second.into();
		// The firmware works out whether daylight saving applies
		tm.tm_isdst = -1;
		Self::from_time_t(unsafe { sys::mktime(&raw mut tm) })
	}

	/// Milliseconds since the Unix epoch.
	pub fn as_millis(self) -> u64 {
		self.millis
	}

	/// Whole seconds since the Unix epoch.
	pub fn seconds(self) -> u64 {
		self.millis / 1000
	}
}

/// A date and time of day, local or UTC, as the firmware breaks a [`Timestamp`] down. Only the
/// firmware makes them, so each field is in range.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DateTime(sys::tm);

impl DateTime {
	/// # Safety
	/// `tm` must come from the firmware, such as the tick handler's.
	pub unsafe fn from_raw(tm: sys::tm) -> Self {
		Self(tm)
	}

	pub fn year(&self) -> i32 {
		self.0.tm_year + 1900
	}

	pub fn month(&self) -> Month {
		Month::ALL[self.0.tm_mon as usize % 12]
	}

	/// The day of the month, from 1.
	pub fn day(&self) -> u8 {
		self.0.tm_mday as u8
	}

	pub fn weekday(&self) -> Weekday {
		Weekday::ALL[self.0.tm_wday as usize % 7]
	}

	/// From 1, for 1 January.
	pub fn day_of_year(&self) -> u16 {
		(self.0.tm_yday + 1) as u16
	}

	/// 0 to 23.
	pub fn hour(&self) -> u8 {
		self.0.tm_hour as u8
	}

	pub fn minute(&self) -> u8 {
		self.0.tm_min as u8
	}

	/// 0 to 60, for a leap second.
	pub fn second(&self) -> u8 {
		self.0.tm_sec as u8
	}

	/// Whether daylight saving time applies.
	pub fn is_dst(&self) -> bool {
		self.0.tm_isdst > 0
	}

	/// Seconds east of UTC, daylight saving included.
	pub fn utc_offset(&self) -> i32 {
		self.0.tm_gmtoff
	}

	/// The time zone's abbreviation, such as `CST`, if the firmware has one.
	pub fn zone(&self) -> &CStr {
		// SAFETY: c_char and u8 have the same layout
		let bytes: &[u8; 6] = unsafe { &*(&raw const self.0.tm_zone).cast() };
		CStr::from_bytes_until_nul(bytes).unwrap_or(c"")
	}

	/// The moment this is.
	pub fn timestamp(&self) -> Timestamp {
		let mut tm = self.0;
		Timestamp::from_time_t(unsafe { sys::mktime(&raw mut tm) })
	}

	/// Formatted by a C `strftime` `format`, such as `c"%A %e"` for `Monday 3`, in the watch's
	/// language. Empty if it doesn't fit in `N` bytes, or `format` is 256 bytes or longer.
	pub fn format<const N: usize>(&self, format: &CStr) -> StackText<N> {
		// SAFETY: only the firmware makes a DateTime
		unsafe { strftime(format, &self.0) }
	}

	pub fn as_ptr(&self) -> *const sys::tm {
		&raw const self.0
	}
}

/// The span between two [`Timestamp`]s, in milliseconds. Signed, as setting the watch's clock between
/// two readings can put the later one first. Saturates at about 24 days either way.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDelta {
	millis: i32,
}

impl TimeDelta {
	pub const ZERO: Self = Self { millis: 0 };

	pub const fn from_millis(millis: i32) -> Self {
		Self { millis }
	}

	pub const fn as_millis(self) -> i32 {
		self.millis
	}
}

impl Sub for Timestamp {
	type Output = TimeDelta;

	/// The time from `earlier` to `self`.
	fn sub(self, earlier: Self) -> TimeDelta {
		let millis = if self >= earlier {
			i32::try_from(self.millis - earlier.millis).unwrap_or(i32::MAX)
		} else {
			i32::try_from(earlier.millis - self.millis).map_or(i32::MIN, |m| -m)
		};
		TimeDelta::from_millis(millis)
	}
}

impl Timestamp {
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

impl Add<TimeDelta> for Timestamp {
	type Output = Self;

	fn add(self, duration: TimeDelta) -> Self {
		self.shifted(duration.millis)
	}
}

impl Sub<TimeDelta> for Timestamp {
	type Output = Self;

	fn sub(self, duration: TimeDelta) -> Self {
		self.shifted(duration.millis.saturating_neg())
	}
}

impl AddAssign<TimeDelta> for Timestamp {
	fn add_assign(&mut self, duration: TimeDelta) {
		*self = *self + duration;
	}
}

impl SubAssign<TimeDelta> for Timestamp {
	fn sub_assign(&mut self, duration: TimeDelta) {
		*self = *self - duration;
	}
}

impl Add for TimeDelta {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self::from_millis(self.millis.saturating_add(other.millis))
	}
}

impl Sub for TimeDelta {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self::from_millis(self.millis.saturating_sub(other.millis))
	}
}

impl AddAssign for TimeDelta {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl SubAssign for TimeDelta {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

/// The time now, to the millisecond.
pub fn now() -> Timestamp {
	let mut seconds: i32 = 0;
	let mut millis: u16 = 0;
	unsafe { sys::time_ms(&raw mut seconds, &raw mut millis) };
	// it's not before 1970
	Timestamp {
		millis: u64::try_from(seconds).unwrap_or(0) * 1000 + u64::from(millis),
	}
}

/// Midnight at the start of today, local time.
pub fn start_of_today() -> Timestamp {
	Timestamp::from_time_t(unsafe { sys::time_start_of_today() })
}

/// `as u8` gives its number, from 0 for Sunday.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Weekday {
	Sunday,
	Monday,
	Tuesday,
	Wednesday,
	Thursday,
	Friday,
	Saturday,
}

impl Weekday {
	const ALL: [Self; 7] = [
		Self::Sunday,
		Self::Monday,
		Self::Tuesday,
		Self::Wednesday,
		Self::Thursday,
		Self::Friday,
		Self::Saturday,
	];
}

/// `as u8` gives its number, from 1 for January.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Month {
	January = 1,
	February,
	March,
	April,
	May,
	June,
	July,
	August,
	September,
	October,
	November,
	December,
}

impl Month {
	const ALL: [Self; 12] = [
		Self::January,
		Self::February,
		Self::March,
		Self::April,
		Self::May,
		Self::June,
		Self::July,
		Self::August,
		Self::September,
		Self::October,
		Self::November,
		Self::December,
	];
}

/// The next time it's `hour:minute` (24 hour) local time on `day`, or today with `None`: this
/// week's if it's still to come, or a week later if it's passed.
pub fn next(day: Option<Weekday>, hour: u8, minute: u8) -> Timestamp {
	// The firmware's WeekDay is 0 for today, then Sunday from 1
	let day = day.map_or(0, |day| day as sys::WeekDay + 1);
	let t = unsafe { sys::clock_to_timestamp(day, hour.into(), minute.into()) };
	Timestamp::from_time_t(t)
}

/// Whether the user shows the time 24 hour.
pub fn is_24h() -> bool {
	unsafe { sys::clock_is_24h_style() }
}

/// The time as the user shows it, such as `15:00`, or `3:00 PM` in 12 hour time.
pub fn clock_text() -> StackText<8> {
	let mut text = StackText::new();
	text.fill_c(|buf| unsafe {
		sys::clock_copy_time_string(buf.as_mut_ptr().cast(), buf.len() as u8);
	});
	text
}

// The firmware's TIMEZONE_NAME_LENGTH, less its NUL
const TIMEZONE_LEN: usize = sys::TIMEZONE_NAME_LENGTH as usize - 1;

/// The watch's time zone, such as `America/Chicago`, or `None` until the phone has set one.
pub fn timezone() -> Option<StackText<TIMEZONE_LEN>> {
	if !unsafe { sys::clock_is_timezone_set() } {
		return None;
	}
	let mut text = StackText::new();
	// SAFETY: the firmware writes up to TIMEZONE_NAME_LENGTH bytes, whatever size it's given, and buf
	// is that long
	text.fill_c(|buf| unsafe { sys::clock_get_timezone(buf.as_mut_ptr().cast(), buf.len()) });
	Some(text)
}

// The firmware kills the app over a format this long or longer
const STRFTIME_FORMAT_MAX: usize = 256;

/// `time` formatted by `format` into `N` bytes, empty if it doesn't fit.
///
/// # Safety
/// `time` must come from the firmware, such as from `localtime`: it indexes tables by fields such as
/// `tm_wday` without checking them, and copies `tm_zone` up to its NUL.
unsafe fn strftime<const N: usize>(format: &CStr, time: &sys::tm) -> StackText<N> {
	let mut text = StackText::new();
	if format.count_bytes() >= STRFTIME_FORMAT_MAX {
		return text;
	}
	text.fill_c(|buf| {
		// SAFETY: writes at most buf.len() bytes, NUL included, and returns 0 if they don't fit; time
		// is valid, per the caller
		let len = unsafe {
			sys::strftime(
				buf.as_mut_ptr().cast::<c_char>(),
				buf.len(),
				format.as_ptr(),
				time,
			)
		};
		if len == 0 {
			buf[0] = 0;
		}
	});
	text
}
