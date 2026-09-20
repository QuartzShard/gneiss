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

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Sample {
	x: i16,
	y: i16,
	z: i16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AccelData {
	sample:      Sample,
	did_vibrate: bool,
	timestamp:   u64,
}

impl Sample {
	pub fn x(&self) -> i16 {
		self.x
	}

	pub fn y(&self) -> i16 {
		self.y
	}

	pub fn z(&self) -> i16 {
		self.z
	}
}

impl AccelData {
	pub fn sample(&self) -> Sample {
		self.sample
	}

	pub fn x(&self) -> i16 {
		self.sample.x
	}

	pub fn y(&self) -> i16 {
		self.sample.y
	}

	pub fn z(&self) -> i16 {
		self.sample.z
	}

	pub fn did_vibrate(&self) -> bool {
		self.did_vibrate
	}

	pub fn timestamp(&self) -> u64 {
		self.timestamp
	}
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

/// The following From-by-transmute impls are only valid while these all hold.
const _PACKED_VALUE_EQ: () = {
	use core::mem::{align_of, offset_of, size_of};
	assert!(size_of::<Sample>() == size_of::<sys::AccelRawData>());
	assert!(align_of::<Sample>() == align_of::<sys::AccelRawData>());
	assert!(offset_of!(Sample, x) == offset_of!(sys::AccelRawData, x));
	assert!(offset_of!(Sample, y) == offset_of!(sys::AccelRawData, y));
	assert!(offset_of!(Sample, z) == offset_of!(sys::AccelRawData, z));

	assert!(size_of::<AccelData>() == size_of::<sys::AccelData>());
	assert!(align_of::<AccelData>() == align_of::<sys::AccelData>());
	assert!(offset_of!(AccelData, sample) == offset_of!(sys::AccelData, x));
	assert!(offset_of!(AccelData, did_vibrate) == offset_of!(sys::AccelData, did_vibrate));
	assert!(offset_of!(AccelData, timestamp) == offset_of!(sys::AccelData, timestamp));
};

impl From<sys::AccelData> for AccelData {
	fn from(value: sys::AccelData) -> Self {
		unsafe { core::mem::transmute(value) }
	}
}

impl From<sys::AccelRawData> for Sample {
	fn from(value: sys::AccelRawData) -> Self {
		unsafe { core::mem::transmute(value) }
	}
}
