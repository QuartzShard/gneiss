//! Touch screen events, for [`TouchService`](crate::service::TouchService).
use crate::sys;

/// One touch event, at `x`, `y`.
pub struct Event {
	/// What happened.
	pub event_type:       EventType,
	/// The touch began while the watch was idle, such as a stray contact on the watchface, so it
	/// shouldn't drive navigation. Set at touchdown and kept for the whole gesture.
	pub non_navigational: bool,
	pub x:                i16,
	pub y:                i16,
}

/// A finger landing, lifting, or moving.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum EventType {
	TouchDown      = sys::TouchEventType_TouchEvent_Touchdown,
	LiftOff        = sys::TouchEventType_TouchEvent_Liftoff,
	PositionUpdate = sys::TouchEventType_TouchEvent_PositionUpdate,
}

impl From<sys::TouchEvent> for Event {
	fn from(value: sys::TouchEvent) -> Self {
		let event_type = match value.type_() {
			sys::TouchEventType_TouchEvent_Touchdown => EventType::TouchDown,
			sys::TouchEventType_TouchEvent_Liftoff => EventType::LiftOff,
			sys::TouchEventType_TouchEvent_PositionUpdate => EventType::PositionUpdate,
			_ => crate::fail::gneiss_panic!("invalid TouchEvent variant"),
		};
		Self {
			event_type,
			non_navigational: value.non_navigational,
			x: value.x,
			y: value.y,
		}
	}
}
