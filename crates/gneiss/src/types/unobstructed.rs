//! The unobstructed area: the part of the screen system overlays, such as Timeline Quick View,
//! leave uncovered. For [`UnobstructedService`](crate::service::UnobstructedService).
use embedded_graphics_core::primitives::Rectangle;

use crate::{sys, ui::graphics::rect};

/// The unobstructed area changing, as an overlay slides in or out.
pub enum Event {
	WillChange {
		area: Rectangle,
	},
	/// It's moved, `progress` of the way, out of 65 535. Read where it is now with
	/// [`Layer::unobstructed_bounds`](crate::ui::Layer::unobstructed_bounds).
	Changing {
		progress: u16,
	},
	DidChange,
}

impl Event {
	#[doc(hidden)]
	pub fn __will_change(area: sys::GRect) -> Self {
		Self::WillChange { area: rect(area) }
	}

	#[doc(hidden)]
	pub fn __changing(progress: sys::AnimationProgress) -> Self {
		Self::Changing {
			progress: progress.clamp(0, u16::MAX.into()) as u16,
		}
	}
}

bitflags::bitflags! {
/// Which of the changes to report.
pub struct SubscribeTo: u8 {
	const WILL = 1;
	const CHANGING = 1 << 1;
	const DID = 1 << 2;
}}
