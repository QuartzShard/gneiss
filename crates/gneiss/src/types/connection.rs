//! Types for describing connection events

pub enum Event {
	AppConnected,
	AppDisconnected,
	PebbleKitConnected,
	PebbleKitDisconnected,
}

pub struct State {
	pub app:       bool,
	pub pebblekit: bool,
}

bitflags::bitflags! {pub struct SubscribeTo: u8 {
	const APP = 1;
	const PEBBLEKIT = 1 << 1;
}}
