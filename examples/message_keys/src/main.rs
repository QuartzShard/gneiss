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

use gneiss::{
	link::{AppMessageEvent, BufferSizes, Outbox, SubscribeTo},
	prelude::*,
	service::*,
	time::{DateTime, TimeUnits},
	ui::{HeapText, StatusBoard, UiGuard, UiQueue},
};

#[repr(usize)]
enum Slot {
	Sent,
	Recv,
	Event,
}
static BOARD: StatusBoard<3> = StatusBoard::new();

/// Context threaded through the AppMessage handler, to show the `Option<&mut T>`
/// context surviving between callbacks.
struct MsgCtx {
	received: u32,
	sent:     u32,
}

#[gneiss::entry]
fn main(ui: &mut UiGuard) -> UiQueue<'_> {
	BOARD.init();
	BOARD.set_line(ui, 0, "messageKeys demo");
	let mut queue = UiQueue::new(ui);
	queue.push(&BOARD, false);
	queue
}

// --- read side: keys drive get / get_array_slot / iter on the inbox handle ---

#[gneiss::service(
	AppMessageService<MsgCtx>,
	(
		SubscribeTo::RECEIVED | SubscribeTo::SENT | SubscribeTo::FAILED,
        BufferSizes::default(),
		Some(MsgCtx { received: 0, sent: 0 }),
	)
)]
fn on_message(event: AppMessageEvent, ctx: Option<&mut MsgCtx>) {
	let mut guard = UiGuard::enter();
	match event {
		AppMessageEvent::Received { inbox } => {
			let n = ctx.map_or(0, |c| {
				c.received += 1;
				c.received
			});
			let cmd = inbox.get::<u8>(message_keys::Command).ok();
			let first_sample = inbox.get_array_slot::<i32>(message_keys::Samples, 0).ok();
			let tuples = inbox.iter().count();
			BOARD.set_line(
				&mut guard,
				Slot::Recv as usize,
				pbl_fmt_alloc!(
					"Recv #{n}: cmd={} s0={} tuples={tuples}",
					or_none(cmd),
					or_none(first_sample)
				),
			);
		}
		AppMessageEvent::Sent { .. } => {
			let n = ctx.map_or(0, |c| {
				c.sent += 1;
				c.sent
			});
			BOARD.set_line(
				&mut guard,
				Slot::Event as usize,
				pbl_fmt_alloc!("Event: sent {n} ok"),
			);
		}
		AppMessageEvent::Failed { reason, .. } => {
			BOARD.set_line(
				&mut guard,
				Slot::Event as usize,
				pbl_fmt_alloc!("Event: send failed {:#x}", reason.bits()),
			);
		}
		AppMessageEvent::Dropped { reason } => {
			BOARD.set_line(
				&mut guard,
				Slot::Event as usize,
				pbl_fmt_alloc!("Event: inbox dropped {:#x}", reason.bits()),
			);
		}
	}
}

/// `value`, or `none`
fn or_none(value: Option<impl Into<i32>>) -> HeapText {
	match value {
		Some(value) => pbl_fmt_alloc!("{}", value.into()),
		None => "none".into(),
	}
}

// --- write side: build an outbox under the same keys and send it ---

#[gneiss::service(TickService, TimeUnits::SECONDS)]
fn on_tick(time: DateTime, _changed: TimeUnits) {
	// SAFETY: no DictHandle view is live here; the buffers are open (the
	// AppMessage service opened them at registration).
	let Some(mut outbox) = (unsafe { Outbox::open() }) else {
		BOARD.set_line(
			&mut UiGuard::enter(),
			Slot::Sent as usize,
			"Sent: outbox busy",
		);
		return;
	};
	let _ = outbox.set(message_keys::Status, c"tick");
	let _ = outbox.set(message_keys::Uptime, &u32::from(time.second()));
	for i in 0..8 {
		if let Some(slot) = message_keys::Samples.get(i) {
			let _ = outbox.set(slot, &(i as i32));
		}
	}
	BOARD.set_line(
		&mut UiGuard::enter(),
		Slot::Sent as usize,
		pbl_fmt_alloc!("Sent: uptime={}s + 8 samples", time.second()),
	);
	// outbox drops here -> app_message_outbox_send
}
