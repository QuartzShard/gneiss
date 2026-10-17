//! Health tracker events, for [`HealthService`](crate::service::HealthService).
//!
//! TODO: Health system API, service does not cut it
use crate::sys;

/// Which kind of health data has new values.
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
			_ => crate::fail::gneiss_panic!("invalid health Event variant"),
		}
	}
}
