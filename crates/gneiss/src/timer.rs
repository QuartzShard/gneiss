//! Firmware Timer handle, used for driving events.
//!
//! A [`Timer`] calls its closure from the event loop once the timeout passes. Dropping the handle
//! cancels it, so a timer held in a [`Layout`](crate::ui::Layout) stops when its window unloads.
//! [`detach`](Timer::detach) lets one run on without a handle instead.
//!
//! ```no_run
//! # use gneiss::{
//! #     time::{self, Timestamp},
//! #     timer::Timer,
//! #     ui::{Layout, StaticWindow, UiGuard, WindowCtx},
//! # };
//! # static STOPWATCH: StaticWindow<Stopwatch> = StaticWindow::new();
//! # struct Stopwatch {
//! #     since:  Option<Timestamp>,
//! #     redraw: Option<Timer>,
//! # }
//! # impl Layout for Stopwatch {
//! #     fn load(_ctx: &mut WindowCtx<'_>) -> Self {
//! #         Self { since: None, redraw: None }
//! #     }
//! # }
//! const FRAME_MS: u32 = 30;
//!
//! impl Stopwatch {
//!     fn start(&mut self) {
//!         // The clock measures the time. The timer only redraws, so a late frame never skews the
//!         // reading
//!         self.since = Some(time::now());
//!         self.redraw = Some(Timer::repeating(FRAME_MS, || {
//!             STOPWATCH.with(&mut UiGuard::enter(), |stopwatch, _| {
//!                 if let Some(stopwatch) = stopwatch {
//!                     stopwatch.show();
//!                 }
//!             });
//!             // Returning `Some(timeout_ms)` runs it again after that long, `None` stops it
//!             Some(FRAME_MS)
//!         }));
//!     }
//!
//!     fn stop(&mut self) {
//!         // Dropping the timer cancels it, as does dropping the layout when the window unloads
//!         self.redraw = None;
//!     }
//! #   fn show(&mut self) {}
//! }
//! ```
//!
//! The `stopwatch` example is the whole app.

use core::{cell::Cell, mem::ManuallyDrop, ptr};

use crate::{
	sys,
	types::{Callable, CallbackSig, ErasedCallback, ErasedPtr, Sig},
	ui::NoWindow,
};

/// A firmware timer.
/// Takes either an `FnOnce` or `FnMut`, and runs it after a timeout passes. Can be rescheduled
/// internally by calling the method or by returning `Some(new_timeout)` from the `FnMut`.
pub struct Timer(ManuallyDrop<ErasedCallback<'static, TimerSig, TimerHeader>>);

/// A timer callback's signature: no arguments, and the delay before it runs again.
enum TimerSig {}

impl Sig for TimerSig {
	type Args<'a: 'b, 'b> = ();
	type Output = Option<u32>;
}

impl<F: FnMut() -> Option<u32>> Callable<TimerSig> for F {
	fn call<'a: 'b, 'b>(&mut self, (): ()) -> Option<u32> {
		self()
	}
}

struct TimerHeader {
	id:    Cell<*mut sys::AppTimer>,
	flags: Cell<TimerFlags>,
}

bitflags::bitflags! {
	#[derive(Clone, Copy, Default)]
	struct TimerFlags: u8 {
		const RUNNING   = 1;
		const DETACHED = 1 << 1;
		const DROPPED   = 1 << 2;
	}
}

impl Timer {
	fn new<F: FnMut() -> Option<u32> + NoWindow + 'static>(timeout_ms: u32, callback: F) -> Timer {
		let inner = ErasedCallback::new(
			TimerHeader {
				id:    Cell::new(ptr::null_mut()),
				flags: Cell::default(),
			},
			callback,
		);

		let id = unsafe {
			sys::app_timer_register(timeout_ms, Some(inner.trampoline()), inner.as_raw())
		};
		inner.header().id.set(id);

		Timer(ManuallyDrop::new(inner))
	}

	/// Start a new `Timer` that will call `f`, returning `Some(timeout_ms)` to reschedule itself, or None to end
	pub fn repeating<F: FnMut() -> Option<u32> + NoWindow + 'static>(
		timeout_ms: u32,
		f: F,
	) -> Timer {
		Self::new(timeout_ms, f)
	}

	/// Start a new `Timer` that will call `f` once.
	pub fn oneshot<F: FnOnce() + NoWindow + 'static>(timeout_ms: u32, f: F) -> Timer {
		let mut f = Some(f);
		Self::new(timeout_ms, move || {
			if let Some(f) = f.take() {
				f();
			}
			None
		})
	}

	/// Run the callback `timeout_ms` from now.
	/// This will rechedule a pending timer, or restart a stopped one
	/// Takes priority over the Some(timeout) return of a repeating timer
	pub fn reschedule(&mut self, timeout_ms: u32) {
		let header = self.0.header();
		let id = header.id.get();
		if !id.is_null() {
			// SAFETY: id is this timer's live registration
			if unsafe { sys::app_timer_reschedule(id, timeout_ms) } {
				return;
			}
			// It fired and its call is queued: cancel stops that call, then register again
			unsafe { sys::app_timer_cancel(id) };
		}
		let id = unsafe {
			sys::app_timer_register(timeout_ms, Some(self.0.trampoline()), self.0.as_raw())
		};
		header.id.set(id);
	}

	/// True until the timer has fired, but not necessarily after the handler runs
	pub fn is_pending(&self) -> bool {
		!self.0.header().id.get().is_null()
	}

	/// "Background" this timer, will call until it stops itself.
	/// The timer will deallocate when it stops, so no resource leakage occurs
	pub fn detach(self) {
		self.0
			.header()
			.flags
			.set(self.0.header().flags.get() | TimerFlags::DETACHED);

		// If the timer finished running and wasn't re-pended, this would be a leak.
		if self.is_pending() || self.0.header().flags.get().contains(TimerFlags::RUNNING) {
			core::mem::forget(self);
		}
	}
}

impl CallbackSig for TimerSig {
	/// The firmware's callback for timers running an `F`.
	unsafe extern "C" fn trampoline<F: Callable<TimerSig>>(ptr: *mut core::ffi::c_void) {
		// SAFETY: only an ErasedCallback of an F hands this out, with its own allocation as data. The
		// allocation lives until Timer's drop frees it outside a callback, or this frees it as its last act
		let data = unsafe { ErasedPtr::<TimerHeader>::from_raw(ptr) };
		let header = unsafe { data.header() };

		// The timer expired to call us
		header.id.set(ptr::null_mut());

		header.flags.set(header.flags.get() | TimerFlags::RUNNING);
		// SAFETY: nothing else borrows the closure: Timer never calls it, and fire can't reenter
		let rearm = unsafe { data.call::<TimerSig, F>(()) };
		header.flags.set(header.flags.get() & !TimerFlags::RUNNING);

		// Timer handle was dropped during the handler
		if header.flags.get().contains(TimerFlags::DROPPED) {
			if !header.id.get().is_null() {
				unsafe { sys::app_timer_cancel(header.id.get()) }
			}
			unsafe { data.free::<F>() };
			return;
		}

		// Nothing has rescheduled
		if header.id.get().is_null() {
			match rearm {
				Some(duration) => header.id.set(unsafe {
					sys::app_timer_register(duration, Some(Self::trampoline::<F>), ptr)
				}),
				None if header.flags.get().contains(TimerFlags::DETACHED) => unsafe {
					data.free::<F>();
				},
				None => (),
			}
		}
	}
}

impl Drop for Timer {
	fn drop(&mut self) {
		let header = self.0.header();
		// Currently inside the callback, don't invalidate, just track
		let flags = header.flags.get();
		if flags.contains(TimerFlags::RUNNING) {
			header.flags.set(flags | TimerFlags::DROPPED);
		} else {
			// SAFETY: This drop path is the last
			let id = header.id.get();
			if !id.is_null() {
				unsafe { sys::app_timer_cancel(id) };
			}
			unsafe { ManuallyDrop::drop(&mut self.0) };
		}
	}
}
