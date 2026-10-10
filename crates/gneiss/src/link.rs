//! `AppMessage` link layer: subscription flags, the event stream, and the
//! outbound handle. Reads run through [`crate::dict::DictHandle`], which the
//! read events carry directly.
use core::ptr;

use crate::{
	dict::{DictHandle, DictionaryError, SingleKey, ToTuple},
	log, sys,
};

bitflags::bitflags! {
	pub struct SubscribeTo : u8 {
		const RECIEVED = 1;
		const DROPPED = 1 << 1;
		const SENT = 1 << 2;
		const FAILED = 1 << 3;
	}
}

pub enum AppMessageEvent<'msg> {
	Recieved {
		inbox: DictHandle<'msg>,
	},
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
	/// Empty means `APP_MSG_OK`.
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

/// Size in bytes of buffer to request at in/outbox open
/// Will be clamped between min/max, or if None, `sys::app_message_*_box_size_maximum()`,
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
			reason = "Strictly speaking, view must NOT exsit by the time open is called"
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
