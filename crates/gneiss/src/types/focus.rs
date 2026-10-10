//! App focus events, for [`AppFocusService`](crate::service::AppFocusService).

/// The app gaining or losing the screen to something like a notification. `Gaining` and `Losing`
/// come as the transition starts, `Gained` and `Lost` once it's done.
pub enum Event {
	Gaining,
	Losing,
	Gained,
	Lost,
}

bitflags::bitflags! {
/// `WILL` for transitions starting, `DID` for transitions done.
pub struct SubscribeTo: u8 {
	const WILL = 1;
	const DID = 1 << 1;
}}
