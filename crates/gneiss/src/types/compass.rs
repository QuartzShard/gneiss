//! Types for describing headings and angles based on the watch's magnetometer.
use core::ops::{Add, Sub};

use crate::sys;

/// A compass reading, from [`CompassService`](crate::service::CompassService).
pub struct HeadingData {
	magnetic_heading:  Heading,
	true_heading:      Heading,
	status:            CompassStatus,
	declination_valid: bool,
}

impl HeadingData {
	/// Get the heading according to the magnetometer.
	pub fn magnetic_heading(&self) -> Heading {
		self.magnetic_heading
	}

	/// `true_heading` is currently identical to `magnetic_heading` and `declination_valid` is
	/// always false (as of 2026-09) - so the value is not useful. This function will always return
	/// None until a firmware update adds system-side declination.
	pub fn true_heading(&self) -> Option<Heading> {
		self.declination_valid.then_some(self.true_heading)
	}

	/// Returns `true_heading` when available, falling back to `magnetic_heading`.
	/// See notes on [`Self::true_heading`].
	pub fn heading(&self) -> Heading {
		self.true_heading().unwrap_or(self.magnetic_heading)
	}

	/// How far to trust the heading.
	pub fn status(&self) -> CompassStatus {
		self.status
	}
}

/// Represents an angle relative to get to a reference direction, e.g. (magnetic) north.
/// The angle value is scaled linearly, such that a value of `TRIG_MAX_ANGLE` corresponds
/// to 360 degrees or 2 PI radians. Thus, if heading towards north, north is 0, west is
/// `TRIG_MAX_ANGLE`/4, south is `TRIG_MAX_ANGLE`/2, and so on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Heading {
	// [0, FULL_SCALE)
	inner: i32,
}

/// Difference between headings, [-180°, 180°) .
/// Positive is CCW, matching firmware internals
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DeltaAngle {
	// [-FULL_SCALE/2, FULL_SCALE/2)
	inner: i32,
}

const _: () = assert!(sys::TRIG_MAX_ANGLE < i32::MAX as u32);
const _: () = assert!(sys::TRIG_MAX_ANGLE == 0x10000);
impl Heading {
	const FULL_SCALE: u32 = sys::TRIG_MAX_ANGLE;
	const HALF_SCALE: i32 = (sys::TRIG_MAX_ANGLE / 2) as i32;

	/// Creates a new Heading from input in [0, 360), returns None on out-of-range
	pub fn from_degrees(val: i16) -> Option<Self> {
		match val {
			0..360 => Some(Self {
				inner: i32::from(val) * Heading::FULL_SCALE as i32 / 360,
			}),
			_ => None,
		}
	}

	/// Creates a new Heading using firmware scale, returns None on out-of-range
	pub fn from_raw(val: i32) -> Option<Self> {
		match val as u32 {
			0..Self::FULL_SCALE => Some(Self { inner: val }),
			_ => None,
		}
	}

	/// Creates a new Heading using firmware scale, wrapping out-of-range inputs
	pub fn from_raw_wrapping(val: i32) -> Self {
		Self {
			inner: val.rem_euclid(Self::FULL_SCALE as i32),
		}
	}

	/// The raw value, in the firmware's `TRIG_MAX_ANGLE` scale.
	pub fn inner(&self) -> sys::CompassHeading {
		self.inner
	}

	/// Coerces a fully-rotated heading to [0, `FULL_SCALE`)
	fn wrap_scale(self) -> u32 {
		self.inner.rem_euclid(Self::FULL_SCALE as i32) as u32
	}

	/// Clockwise heading in degrees
	pub fn as_degrees_cw(&self) -> u16 {
		(360 - self.as_degrees_ccw()) % 360
	}

	/// Counter-Clockwise heading in degrees
	pub fn as_degrees_ccw(&self) -> u16 {
		(self.wrap_scale() * 360 / Self::FULL_SCALE) as u16
	}

	/// Clockwise heading in steps of 1 / 2^16
	pub fn cw(&self) -> u16 {
		(Self::FULL_SCALE - u32::from(self.ccw())) as u16 // Performs wrapping implicitly
	}

	/// Counter-Clockwise heading in steps of 1 / 2^16
	pub fn ccw(&self) -> u16 {
		self.wrap_scale() as u16
	}
}

impl DeltaAngle {
	/// Creates a new `DeltaAngle` from input in [-180, 180), returns None on out-of-range
	pub fn from_degrees(val: i16) -> Option<Self> {
		match val {
			-180..180 => Some(Self {
				inner: i32::from(val) * Heading::FULL_SCALE as i32 / 360,
			}),
			_ => None,
		}
	}

	/// Creates a new `DeltaAngle` in [`HALF_SCALE`, `HALF_SCALE`), returns None on out-of-range
	pub fn from_raw(val: i32) -> Option<Self> {
		let half_scale_range = -Heading::HALF_SCALE..Heading::HALF_SCALE;
		if half_scale_range.contains(&val) {
			Some(DeltaAngle { inner: val })
		} else {
			None
		}
	}

	/// Creates a new `DeltaAngle` in [-`HALF_SCALE`, `HALF_SCALE`), wrapping out-of-range inputs.
	pub fn from_raw_wrapping(val: i32) -> Self {
		Self {
			inner: (val + Heading::HALF_SCALE).rem_euclid(Heading::FULL_SCALE as i32)
				- Heading::HALF_SCALE,
		}
	}

	/// The raw value, in the firmware's `TRIG_MAX_ANGLE` scale.
	pub fn inner(&self) -> sys::CompassHeading {
		self.inner
	}

	/// Coerces a fully-rotated heading to [-`FULL_SCALE`/2, `FULL_SCALE`/2)
	fn wrap_scale(self) -> i32 {
		(self.inner + Heading::HALF_SCALE).rem_euclid(Heading::FULL_SCALE as i32)
			- Heading::HALF_SCALE
	}

	/// Counter-Clockwise heading in degrees
	pub fn as_degrees_ccw(&self) -> i16 {
		(self.wrap_scale() * 360 / Heading::FULL_SCALE as i32) as i16
	}

	/// Clockwise heading in degrees
	pub fn as_degrees_cw(&self) -> i16 {
		360 - self.as_degrees_ccw()
	}
}

impl Add<DeltaAngle> for Heading {
	type Output = Heading;

	fn add(self, rhs: DeltaAngle) -> Self::Output {
		let mut out = Heading {
			inner: self.inner + rhs.inner(),
		};
		out.inner = out.wrap_scale() as i32;
		out
	}
}

impl Sub<DeltaAngle> for Heading {
	type Output = Heading;

	fn sub(self, rhs: DeltaAngle) -> Self::Output {
		let mut out = Heading {
			inner: self.inner - rhs.inner(),
		};
		out.inner = out.wrap_scale() as i32;
		out
	}
}

impl Sub<Heading> for Heading {
	type Output = DeltaAngle;

	fn sub(self, rhs: Heading) -> Self::Output {
		let mut out = DeltaAngle {
			inner: self.inner - rhs.inner(),
		};
		out.inner = out.wrap_scale();
		out
	}
}

/// How trustworthy a compass reading is. Headings are only accurate once `Calibrated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompassStatus {
	Unavailable,
	DataInvalid,
	Calibrating,
	Calibrated,
}

impl From<sys::CompassHeadingData> for HeadingData {
	fn from(value: sys::CompassHeadingData) -> Self {
		HeadingData {
			magnetic_heading:  Heading::from_raw_wrapping(value.magnetic_heading),
			true_heading:      Heading::from_raw_wrapping(value.true_heading),
			status:            CompassStatus::from(value.compass_status),
			declination_valid: value.is_declination_valid,
		}
	}
}

impl From<sys::CompassStatus> for CompassStatus {
	fn from(value: sys::CompassStatus) -> Self {
		match value {
			sys::CompassStatus_CompassStatusUnavailable => Self::Unavailable,
			sys::CompassStatus_CompassStatusDataInvalid => Self::DataInvalid,
			sys::CompassStatus_CompassStatusCalibrating => Self::Calibrating,
			sys::CompassStatus_CompassStatusCalibrated => Self::Calibrated,
			_other => crate::fail::gneiss_panic!("invalid CompassStatus variant"),
		}
	}
}

impl From<CompassStatus> for sys::CompassStatus {
	fn from(value: CompassStatus) -> Self {
		match value {
			CompassStatus::Unavailable => sys::CompassStatus_CompassStatusUnavailable,
			CompassStatus::DataInvalid => sys::CompassStatus_CompassStatusDataInvalid,
			CompassStatus::Calibrating => sys::CompassStatus_CompassStatusCalibrating,
			CompassStatus::Calibrated => sys::CompassStatus_CompassStatusCalibrated,
		}
	}
}
