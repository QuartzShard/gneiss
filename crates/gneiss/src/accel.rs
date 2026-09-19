//! Types for interfacing with the watch's accelorometer
use crate::sys;

pub enum Axis {
	X,
	Y,
	Z,
}

pub enum Direction {
	Positive,
	Negative,
}

impl From<sys::AccelAxisType> for Axis {
	fn from(value: sys::AccelAxisType) -> Self {
		match value {
			sys::AccelAxisType_ACCEL_AXIS_X => Self::X,
			sys::AccelAxisType_ACCEL_AXIS_Y => Self::Y,
			sys::AccelAxisType_ACCEL_AXIS_Z => Self::Z,
			#[expect(
				clippy::panic,
				reason = "FFI Boundary: Value outside of known variants"
			)]
			_ => panic!("Invalid Axis variant"),
		}
	}
}

impl From<Axis> for sys::AccelAxisType {
	fn from(value: Axis) -> Self {
		match value {
			Axis::X => sys::AccelAxisType_ACCEL_AXIS_X,
			Axis::Y => sys::AccelAxisType_ACCEL_AXIS_Y,
			Axis::Z => sys::AccelAxisType_ACCEL_AXIS_Z,
		}
	}
}

impl From<i32> for Direction {
	fn from(value: i32) -> Self {
		if value.is_positive() {
			Self::Positive
		} else {
			Self::Negative
		}
	}
}

impl From<Direction> for i32 {
	fn from(value: Direction) -> Self {
		match value {
			Direction::Positive => 1,
			Direction::Negative => -1,
		}
	}
}
