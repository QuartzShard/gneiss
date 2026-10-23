//! Types for describing headings and angles based on the watch's magnetometer.

pub use super::angle::{Angle, DeltaAngle};
use crate::sys;

/// How far the watch has turned counter-clockwise from north.
pub type Heading = Angle;

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
