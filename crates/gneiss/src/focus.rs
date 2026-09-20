//! Types for describing focus events
use crate::sys;

pub enum Event {
	Gaining,
	Losing,
	Gained,
	Lost,
}

bitflags::bitflags! {pub struct SubscribeTo: u8 {
	const WILL = 1;
	const DID = 1 << 1;
}}
