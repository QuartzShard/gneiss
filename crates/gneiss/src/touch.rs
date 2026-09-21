use crate::sys;

pub struct Event {
	pub event_type:       EventType,
	pub non_navigational: bool,
	pub x:                i16,
	pub y:                i16,
}

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
			#[expect(
				clippy::panic,
				reason = "FFI Boundary: Value outside of known variants"
			)]
			_ => panic!("Invalid TouchEvent variant"),
		};
		Self {
			event_type,
			non_navigational: value.non_navigational,
			x: value.x,
			y: value.y,
		}
	}
}
