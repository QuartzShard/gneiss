//! Angles in the firmware's scale, which the compass and drawing share.
use core::ops::{Add, Sub};

use crate::sys;

/// An angle in the firmware's scale, where `0x10000` (`TRIG_MAX_ANGLE`) is a full turn.
///
/// The compass reads it as how far the watch has turned counter-clockwise from north, a
/// [`Heading`](super::compass::Heading): west is a quarter turn. Drawing reads the same value as a
/// clockwise rotation, so drawing something rotated by a heading points it at north. The
/// `heading_` accessors read it as the compass does, the others as drawing does.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Angle {
	// [0, FULL_SCALE)
	inner: i32,
}

/// The difference between two [`Angle`]s, in [-180°, 180°). Positive is the compass's
/// counter-clockwise, and a clockwise rotation when drawing.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DeltaAngle {
	// [-FULL_SCALE/2, FULL_SCALE/2)
	inner: i32,
}

const _: () = assert!(sys::TRIG_MAX_ANGLE < i32::MAX as u32);
const _: () = assert!(sys::TRIG_MAX_ANGLE == 0x10000);
impl Angle {
	const FULL_SCALE: u32 = sys::TRIG_MAX_ANGLE;
	const HALF_SCALE: i32 = (sys::TRIG_MAX_ANGLE / 2) as i32;

	/// Creates a new Angle from input in [0, 360), returns None on out-of-range
	pub fn from_degrees(val: i16) -> Option<Self> {
		match val {
			0..360 => Some(Self {
				inner: i32::from(val) * Angle::FULL_SCALE as i32 / 360,
			}),
			_ => None,
		}
	}

	/// Creates a new Angle using firmware scale, returns None on out-of-range
	pub fn from_raw(val: i32) -> Option<Self> {
		match val as u32 {
			0..Self::FULL_SCALE => Some(Self { inner: val }),
			_ => None,
		}
	}

	/// Creates a new Angle using firmware scale, wrapping out-of-range inputs
	pub fn from_raw_wrapping(val: i32) -> Self {
		Self {
			inner: val.rem_euclid(Self::FULL_SCALE as i32),
		}
	}

	/// The raw value in the firmware's `TRIG_MAX_ANGLE` scale.
	pub fn inner(&self) -> i32 {
		self.inner
	}

	/// Coerces a fully-rotated angle to [0, `FULL_SCALE`)
	fn wrap_scale(self) -> u32 {
		self.inner.rem_euclid(Self::FULL_SCALE as i32) as u32
	}

	/// As a heading, clockwise, in degrees
	pub fn heading_degrees_cw(&self) -> u16 {
		(360 - self.heading_degrees_ccw()) % 360
	}

	/// As a heading, counter-clockwise, in degrees
	pub fn heading_degrees_ccw(&self) -> u16 {
		(self.wrap_scale() * 360 / Self::FULL_SCALE) as u16
	}

	/// As a heading, clockwise, in steps of 1 / 2^16
	pub fn heading_cw(&self) -> u16 {
		(Self::FULL_SCALE - u32::from(self.heading_ccw())) as u16 // Performs wrapping implicitly
	}

	/// As a heading, counter-clockwise, in steps of 1 / 2^16
	pub fn heading_ccw(&self) -> u16 {
		self.wrap_scale() as u16
	}

	/// As a drawing rotation, clockwise, in degrees
	pub fn degrees_cw(&self) -> u16 {
		self.heading_degrees_ccw()
	}

	/// As a drawing rotation, counter-clockwise, in degrees
	pub fn degrees_ccw(&self) -> u16 {
		self.heading_degrees_cw()
	}

	/// As a drawing rotation, clockwise, in steps of 1 / 2^16
	pub fn cw(&self) -> u16 {
		self.heading_ccw()
	}

	/// As a drawing rotation, counter-clockwise, in steps of 1 / 2^16
	pub fn ccw(&self) -> u16 {
		self.heading_cw()
	}
}

impl DeltaAngle {
	/// Creates a new `DeltaAngle` from input in [-180, 180), returns None on out-of-range
	pub fn from_degrees(val: i16) -> Option<Self> {
		match val {
			-180..180 => Some(Self {
				inner: i32::from(val) * Angle::FULL_SCALE as i32 / 360,
			}),
			_ => None,
		}
	}

	/// Creates a new `DeltaAngle` in [`HALF_SCALE`, `HALF_SCALE`), returns None on out-of-range
	pub fn from_raw(val: i32) -> Option<Self> {
		let half_scale_range = -Angle::HALF_SCALE..Angle::HALF_SCALE;
		if half_scale_range.contains(&val) {
			Some(DeltaAngle { inner: val })
		} else {
			None
		}
	}

	/// Creates a new `DeltaAngle` in [-`HALF_SCALE`, `HALF_SCALE`), wrapping out-of-range inputs.
	pub fn from_raw_wrapping(val: i32) -> Self {
		Self {
			inner: (val + Angle::HALF_SCALE).rem_euclid(Angle::FULL_SCALE as i32)
				- Angle::HALF_SCALE,
		}
	}

	/// The raw value, in the firmware's `TRIG_MAX_ANGLE` scale.
	pub fn inner(&self) -> i32 {
		self.inner
	}

	/// Coerces a fully-rotated angle to [-`FULL_SCALE`/2, `FULL_SCALE`/2)
	fn wrap_scale(self) -> i32 {
		(self.inner + Angle::HALF_SCALE).rem_euclid(Angle::FULL_SCALE as i32) - Angle::HALF_SCALE
	}

	/// As a heading, counter-clockwise, in degrees
	pub fn heading_degrees_ccw(&self) -> i16 {
		(self.wrap_scale() * 360 / Angle::FULL_SCALE as i32) as i16
	}

	/// As a heading, clockwise, in degrees
	pub fn heading_degrees_cw(&self) -> i16 {
		-self.heading_degrees_ccw()
	}

	/// As a drawing rotation, clockwise, in degrees
	pub fn degrees_cw(&self) -> i16 {
		self.heading_degrees_ccw()
	}

	/// As a drawing rotation, counter-clockwise, in degrees
	pub fn degrees_ccw(&self) -> i16 {
		self.heading_degrees_cw()
	}
}

impl Add<DeltaAngle> for Angle {
	type Output = Angle;

	fn add(self, rhs: DeltaAngle) -> Self::Output {
		let mut out = Angle {
			inner: self.inner + rhs.inner(),
		};
		out.inner = out.wrap_scale() as i32;
		out
	}
}

impl Sub<DeltaAngle> for Angle {
	type Output = Angle;

	fn sub(self, rhs: DeltaAngle) -> Self::Output {
		let mut out = Angle {
			inner: self.inner - rhs.inner(),
		};
		out.inner = out.wrap_scale() as i32;
		out
	}
}

impl Sub<Angle> for Angle {
	type Output = DeltaAngle;

	fn sub(self, rhs: Angle) -> Self::Output {
		let mut out = DeltaAngle {
			inner: self.inner - rhs.inner(),
		};
		out.inner = out.wrap_scale();
		out
	}
}
