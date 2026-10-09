//! Types for health service events
use crate::sys;

#[repr(u8)]
pub enum Event {
	Significant = sys::HealthEventType_HealthEventSignificantUpdate,
	Movement    = sys::HealthEventType_HealthEventMovementUpdate,
	Sleep       = sys::HealthEventType_HealthEventSleepUpdate,
	HeartRate   = sys::HealthEventType_HealthEventHeartRateUpdate,
	HRV         = sys::HealthEventType_HealthEventHRVUpdate,
}

impl From<sys::HealthEventType> for Event {
	fn from(value: sys::HealthEventType) -> Self {
		match value {
			sys::HealthEventType_HealthEventSignificantUpdate => Self::Significant,
			sys::HealthEventType_HealthEventMovementUpdate => Self::Movement,
			sys::HealthEventType_HealthEventSleepUpdate => Self::Sleep,
			sys::HealthEventType_HealthEventHeartRateUpdate => Self::HeartRate,
			sys::HealthEventType_HealthEventHRVUpdate => Self::HRV,
			#[expect(
				clippy::panic,
				reason = "FFI Boundary: Value outside of known variants"
			)]
			_ => panic!("Invalid Event variant"),
		}
	}
}
