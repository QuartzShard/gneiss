//! Phone connection events, for [`ConnectionService`](crate::service::ConnectionService).

/// The phone app, or a PebbleKit companion app, connected or disconnected.
pub enum Event {
	AppConnected,
	AppDisconnected,
	PebbleKitConnected,
	PebbleKitDisconnected,
}

/// What's connected right now.
pub struct State {
	/// The Pebble phone app.
	pub app:       bool,
	/// A PebbleKit companion app.
	pub pebblekit: bool,
}

bitflags::bitflags! {
/// Which connections to report on.
pub struct SubscribeTo: u8 {
	const APP = 1;
	const PEBBLEKIT = 1 << 1;
}}
