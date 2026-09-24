#![no_std]
#![no_main]
//! messageKeys demo: keys declared in Cargo.toml become typed `SingleKey` /
//! `ArrayKey` consts in the generated `message_keys` module, which feed the safe
//! AppMessage API directly.
//!
//! - reading: `AppMessageService` hands a `DictHandle`; `get`, `get_array_slot`
//!   and `iter` pull typed values out by key.
//! - writing: `Outbox::open` + `set` build a reply/message under the same keys.
extern crate alloc;

use alloc::format;

use gneiss::{
	display::SimpleTextDisplay,
	link::{AppMessageEvent, BufferSizes, Outbox, SubscribeTo},
	service::*,
	sys,
	time::{TickTime, TimeUnits},
};

#[repr(usize)]
enum Slot {
	Sent,
	Recv,
	Event,
}
static BOARD: SimpleTextDisplay<3> = SimpleTextDisplay::new();

/// Context threaded through the AppMessage handler, to show the `Option<&mut T>`
/// context surviving between callbacks.
struct MsgCtx {
	received: u32,
	sent:     u32,
}

#[gneiss::entry]
fn main() {
	unsafe {
		let window = sys::window_create();
		let root = sys::window_get_root_layer(window);
		let bounds = sys::layer_get_bounds(root);
		let text = sys::text_layer_create(bounds);
		sys::text_layer_set_text_alignment(text, sys::GTextAlignment_GTextAlignmentCenter);
		sys::text_layer_set_font(
			text,
			sys::fonts_get_system_font(sys::FONT_KEY_GOTHIC_18.as_ptr()),
		);
		sys::text_layer_set_text(text, c"messageKeys demo".as_ptr());
		sys::layer_add_child(root, sys::text_layer_get_layer(text));
		sys::window_stack_push(window, false);
		BOARD.attach(text);
	}
}

// --- read side: keys drive get / get_array_slot / iter on the inbox handle ---

#[gneiss::service(
	AppMessageService<MsgCtx>,
	(
		SubscribeTo::RECIEVED | SubscribeTo::SENT | SubscribeTo::FAILED,
        BufferSizes::default(),
		Some(MsgCtx { received: 0, sent: 0 }),
	)
)]
fn on_message(event: AppMessageEvent, ctx: Option<&mut MsgCtx>) {
	match event {
		AppMessageEvent::Recieved { inbox } => {
			let n = ctx.map_or(0, |c| {
				c.received += 1;
				c.received
			});
			let cmd = inbox.get::<u8>(message_keys::Command).ok();
			let first_sample = inbox.get_array_slot::<i32>(message_keys::Samples, 0).ok();
			let tuples = inbox.iter().count();
			BOARD.set(
				Slot::Recv as usize,
				format!("Recv #{n}: cmd={cmd:?} s0={first_sample:?} tuples={tuples}"),
			);
		}
		AppMessageEvent::Sent { .. } => {
			let n = ctx.map_or(0, |c| {
				c.sent += 1;
				c.sent
			});
			BOARD.set(Slot::Event as usize, format!("Event: sent {n} ok"));
		}
		AppMessageEvent::Failed { reason, .. } => {
			BOARD.set(
				Slot::Event as usize,
				format!("Event: send failed {reason:#x}"),
			);
		}
		AppMessageEvent::Dropped { reason } => {
			BOARD.set(
				Slot::Event as usize,
				format!("Event: inbox dropped {reason:#x}"),
			);
		}
	}
}

// --- write side: build an outbox under the same keys and send it ---

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(time: TickTime, _changed: TimeUnits) {
	// SAFETY: no DictHandle view is live here; the buffers are open (the
	// AppMessage service opened them at registration).
	let Some(mut outbox) = (unsafe { Outbox::open() }) else {
		BOARD.set(Slot::Sent as usize, "Sent: outbox busy");
		return;
	};
	let _ = outbox.set(message_keys::Status, c"tick");
	let _ = outbox.set(message_keys::Uptime, &(time.tm_sec as u32));
	for i in 0..8 {
		if let Some(slot) = message_keys::Samples.get(i) {
			let _ = outbox.set(slot, &(i as i32));
		}
	}
	BOARD.set(
		Slot::Sent as usize,
		format!("Sent: uptime={}s + 8 samples", time.tm_sec),
	);
	// outbox drops here -> app_message_outbox_send
}
