//! AppMessage: key-value messages exchanged with the phone.
//!
//! AppMessage carries small dictionaries between the watch app and its companion on the phone,
//! whether that's PebbleKit JS or a native app. Keys are numbers on the wire, not self-describing
//! like, say, JSON. You name them in `Cargo.toml`, and the build generates a `message_keys`
//! module of typed keys:
//!
//! ```toml
//! [package.metadata.pebble]
//! message_keys = ["Command", "Status", "Minute", "Samples[8]"]
//! ```
//!
//! A plain name becomes a [`SingleKey`]; `Name[N]` becomes an [`ArrayKey`](crate::dict::ArrayKey)
//! for `N` consecutive keys. The bundle's `appinfo.json` lists the same names, so the companion can
//! use them too. Values are read and written through [`dict`](crate::dict).
//!
//! # Receiving
//!
//! Subscribe [`AppMessageService`](crate::service::AppMessageService), choosing the events to
//! deliver with [`SubscribeTo`] and the buffer sizes with [`BufferSizes`]. Each message arrives as
//! an [`AppMessageEvent::Received`], carrying a [`DictHandle`] to read from:
//!
//! ```no_run,standalone_crate
//! # use gneiss::{link::*, service::*};
//! # gneiss::__doctest_slices!();
//! # mod message_keys { pub const Command: gneiss::dict::SingleKey = gneiss::dict::SingleKey::new(0); }
//! #[gneiss::service(AppMessageService<()>, SubscribeTo::RECEIVED, BufferSizes::default(), None)]
//! fn on_message(event: AppMessageEvent, _ctx: Option<&mut ()>) {
//!     if let AppMessageEvent::Received { inbox } = event {
//!         let command = inbox.get::<u8>(message_keys::Command);
//!     }
//! }
//! ```
//!
//! # Sending
//!
//! [`Outbox`] builds a message, which sends when it drops. Only one can be open at a time, so
//! opening fails while a message is still in flight, until its [`AppMessageEvent::Sent`] or
//! [`AppMessageEvent::Failed`] arrives.
//!
//! ```no_run,standalone_crate
//! # use gneiss::{link::*, service::*, time::*};
//! # gneiss::__doctest_slices!();
//! # mod message_keys { use gneiss::dict::SingleKey; pub const Status: SingleKey = SingleKey::new(1); pub const Minute: SingleKey = SingleKey::new(2); }
//! #[gneiss::service(TickService, TimeUnits::MINUTES)]
//! fn on_minute(time: TickTime, _changed: TimeUnits) {
//!     // SAFETY: no DictHandle is live outside the AppMessage handler
//!     let Some(mut outbox) = (unsafe { Outbox::open() }) else {
//!         return; // the last message is still in flight
//!     };
//!     let _ = outbox.set(message_keys::Status, c"tick");
//!     let _ = outbox.set(message_keys::Minute, &time.tm_min);
//! } // sends here
//! ```
//!
//! [`Outbox::open`] is `unsafe` because opening reuses the dictionary buffer, so no
//! [`DictHandle`] may still be reading it. Inside the AppMessage handler, reply with
//! [`Outbox::open_after`], which takes the handle you're done with.
use core::ptr;

use crate::{
	dict::{DictHandle, DictionaryError, SingleKey, ToTuple},
	log, sys,
};

bitflags::bitflags! {
	/// Which AppMessage events to deliver.
	pub struct SubscribeTo : u8 {
		const RECEIVED = 1;
		const DROPPED = 1 << 1;
		const SENT = 1 << 2;
		const FAILED = 1 << 3;
	}
}

/// Something that happened to an incoming or outgoing message.
pub enum AppMessageEvent<'msg> {
	Received {
		inbox: DictHandle<'msg>,
	},
	/// A message arrived but couldn't be delivered, e.g. the inbox was too small.
	Dropped {
		reason: AppMessageError,
	},
	Sent {
		outbox: DictHandle<'msg>,
	},
	Failed {
		outbox: DictHandle<'msg>,
		reason: AppMessageError,
	},
}
impl From<sys::AppMessageResult> for AppMessageError {
	fn from(value: sys::AppMessageResult) -> Self {
		Self::from_bits_retain(value)
	}
}

bitflags::bitflags! {
	/// Why AppMessage failed. Empty means `APP_MSG_OK`.
	pub struct AppMessageError : u16 {
		const SEND_TIMEOUT = sys::AppMessageResult_APP_MSG_SEND_TIMEOUT;
		const SEND_REJECTED = sys::AppMessageResult_APP_MSG_SEND_REJECTED;
		const NOT_CONNECTED = sys::AppMessageResult_APP_MSG_NOT_CONNECTED;
		const APP_NOT_RUNNING = sys::AppMessageResult_APP_MSG_APP_NOT_RUNNING;
		const INVALID_ARGS = sys::AppMessageResult_APP_MSG_INVALID_ARGS;
		const BUSY = sys::AppMessageResult_APP_MSG_BUSY;
		const BUFFER_OVERFLOW = sys::AppMessageResult_APP_MSG_BUFFER_OVERFLOW;
		const ALREADY_RELEASED = sys::AppMessageResult_APP_MSG_ALREADY_RELEASED;
		const CALLBACK_ALREADY_REGISTERED = sys::AppMessageResult_APP_MSG_CALLBACK_ALREADY_REGISTERED;
		const CALLBACK_NOT_REGISTERED = sys::AppMessageResult_APP_MSG_CALLBACK_NOT_REGISTERED;
		const OUT_OF_MEMORY = sys::AppMessageResult_APP_MSG_OUT_OF_MEMORY;
		const CLOSED = sys::AppMessageResult_APP_MSG_CLOSED;
		const INTERNAL_ERROR = sys::AppMessageResult_APP_MSG_INTERNAL_ERROR;
		const INVALID_STATE = sys::AppMessageResult_APP_MSG_INVALID_STATE;
	}
}

/// Inbox and outbox sizes in bytes. Each is clamped to what the firmware allows, and `None` asks
/// for the most it allows.
#[derive(Default, Clone, Copy)]
pub struct BufferSizes {
	pub inbox:  Option<u32>,
	pub outbox: Option<u32>,
}

/// The outbound dictionary; sends on drop (or explicit [`Self::send`]).
pub struct Outbox<'msg> {
	handle: DictHandle<'msg>,
}

impl<'msg> Outbox<'msg> {
	/// Open the outbox after finishing with a read-only view, consuming it so no
	/// reference into that view survives the `begin` this triggers.
	/// Return will always be `Some` in the Sent/Failed handlers
	pub fn open_after(view: DictHandle<'msg>) -> Option<Outbox<'msg>> {
		#[allow(
			clippy::drop_non_drop,
			reason = "Strictly speaking, view must NOT exist by the time open is called"
		)]
		drop(view);
		unsafe { Self::open() }
	}

	/// Open the outbox. `None` if it is already open (there cannot be two live
	/// writers on the one dict).
	/// # Safety
	/// Must not be called while any reference into the outbox dictionary (a
	/// `DictHandle` view or its `Refs`) is live: `begin` invalidates it. The safe
	/// [`Self::open_after`] consumes the view for you.
	pub unsafe fn open() -> Option<Outbox<'msg>> {
		// SAFETY: begin() doesn't deref dict_ptr, it replaces it with a pointer to the outbox
		let mut dict_ptr = ptr::null_mut();
		let result = AppMessageError::from_bits_retain(unsafe {
			sys::app_message_outbox_begin(&raw mut dict_ptr)
		});
		if result.is_empty() {
			// SAFETY: APP_MSG_OK, so dict_ptr is a valid outbox iterator for 'msg.
			(!dict_ptr.is_null()).then(|| Outbox {
				handle: unsafe { DictHandle::from_raw(dict_ptr) },
			})
		} else if result.intersects(AppMessageError::BUSY | AppMessageError::INVALID_STATE) {
			log::warn!("Outbox already open");
			None
		} else {
			unsafe {
				log::log_sys_fmt!(
					log::LogLevel::Error,
					"Unexpected response opening outbox: 0x%x",
					core::ffi::c_uint::from(result.bits())
				);
			};
			None
		}
	}

	/// Calls `app_message_outbox_send`.
	/// Equivalent to just dropping it.
	pub fn send(self) {}

	/// Write a value under `key`. Errors carry the raw `DictionaryResult`.
	pub fn set<T: ToTuple + ?Sized>(
		&mut self,
		key: SingleKey,
		value: &T,
	) -> Result<(), DictionaryError> {
		self.handle.set(key, value)
	}
}

impl Drop for Outbox<'_> {
	fn drop(&mut self) {
		// SAFETY: sends the built message; the handle is not used afterward.
		let res = unsafe { AppMessageError::from_bits_retain(sys::app_message_outbox_send()) };
		if !res.is_empty() {
			unsafe {
				log::log_sys_fmt!(
					log::LogLevel::Error,
					"Error on outbox close: 0x%x",
					core::ffi::c_int::from(res.bits())
				);
			}
		}
	}
}
