//! Per-button click handlers for a window.
//!
//! # Binary size
//!
//! A [`ClickHandler`] stores its closure as an `ErasedFn`, not as a `Box<dyn FnMut>`. A trait
//! object's vtable sits in `.data`, needs a load-time relocation per function pointer, and brings a
//! `call_once` shim that is never called: measured at about 52 bytes per registered handler.
//! Function pointers written at runtime need no relocation.
//!
//! The click trampolines are generic over the click kind only, not the window type, so an app has
//! one set however many windows it has.
use core::ffi;

use crate::{
	sys,
	types::{Callable, ErasedFn, Sig},
	ui::{
		Layout, NoWindow, UiGuard, UiQueue, Window,
		window::{Header, WindowData, header},
	},
};

/// A window's click handler for each button.
pub struct ClickHandlers {
	up:              Option<ClickHandler>,
	select:          Option<ClickHandler>,
	down:            Option<ClickHandler>,
	back:            Option<ClickHandler>,
	// Back has been subscribed to SINGLE or MULTI at some point: the firmware's default Back pop
	// never comes back after that
	back_overridden: bool,
}

impl ClickHandlers {
	pub(super) const fn new() -> Self {
		Self {
			up:              None,
			select:          None,
			down:            None,
			back:            None,
			back_overridden: false,
		}
	}

	fn get_mut(&mut self, button: Button) -> &mut Option<ClickHandler> {
		match button {
			Button::Back => &mut self.back,
			Button::Up => &mut self.up,
			Button::Select => &mut self.select,
			Button::Down => &mut self.down,
		}
	}
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Button {
	Back   = sys::ButtonId_BUTTON_ID_BACK,
	Up     = sys::ButtonId_BUTTON_ID_UP,
	Select = sys::ButtonId_BUTTON_ID_SELECT,
	Down   = sys::ButtonId_BUTTON_ID_DOWN,
}

impl Button {
	const ALL: [Self; 4] = [Self::Back, Self::Up, Self::Select, Self::Down];

	pub(crate) fn from_raw(id: sys::ButtonId) -> Option<Self> {
		match id {
			sys::ButtonId_BUTTON_ID_BACK => Some(Self::Back),
			sys::ButtonId_BUTTON_ID_UP => Some(Self::Up),
			sys::ButtonId_BUTTON_ID_SELECT => Some(Self::Select),
			sys::ButtonId_BUTTON_ID_DOWN => Some(Self::Down),
			_ => None,
		}
	}
}

bitflags::bitflags! {
	/// Click recognizer slots to subscribe a button's handler to. Each flag delivers the
	/// matching [`Click`] variant.
	///
	/// Subscribing `MULTI` delays `SINGLE` on that button until the multi-click timeout passes.
	/// `LONG`/`LONG_RELEASE` and `RAW_DOWN`/`RAW_UP` are each registered by one firmware call.
	pub struct ClickKinds: u8 {
		const SINGLE       = 1;
		const MULTI        = 1 << 1;
		const LONG         = 1 << 2;
		const LONG_RELEASE = 1 << 3;
		const RAW_DOWN     = 1 << 4;
		const RAW_UP       = 1 << 5;
	}
}

/// A recognized click on a button, delivered to that button's handler.
pub enum Click {
	Single {
		/// Fired by holding the button down, not by a press.
		repeating: bool,
	},
	Multi {
		/// How many presses were counted.
		count: u8,
	},
	Long,
	LongRelease,
	RawDown,
	RawUp,
}

/// One button's handler, and the clicks it is subscribed to.
pub struct ClickHandler {
	subscribed: ClickKinds,
	// Not a Box<dyn FnMut> on purpose: see the module docs, "Binary size"
	fun:        ErasedFn<'static, ClickSig>,
}

/// A click handler's signature: the window's header, the click, and the navigation queue.
enum ClickSig {}

impl Sig for ClickSig {
	type Args<'a: 'b, 'b> = (*mut Header, Click, &'b mut UiQueue<'a>);
	type Output = ();
}

impl<F: for<'n> FnMut(*mut Header, Click, &mut UiQueue<'n>)> Callable<ClickSig> for F {
	fn call<'a: 'b, 'b>(&mut self, (header, click, queue): <ClickSig as Sig>::Args<'a, 'b>) {
		self(header, click, queue);
	}
}

impl ClickHandler {
	pub(super) fn new<F: for<'n> FnMut(*mut Header, Click, &mut UiQueue<'n>) + 'static>(
		subscribed: ClickKinds,
		closure: F,
	) -> Self {
		Self {
			subscribed,
			fun: ErasedFn::new((), closure),
		}
	}
}

impl<C: Layout> Window<C> {
	/// Replace `button`'s click handler, subscribed to `kinds`. Takes effect immediately if the
	/// window is on screen.
	pub fn set_click_handler(
		&mut self,
		ui: &mut UiGuard,
		button: Button,
		kinds: ClickKinds,
		mut handler: impl for<'n> FnMut(&'n mut C, Option<&'n mut C::Data>, Click, &mut UiQueue<'n>)
		+ NoWindow
		+ 'static,
	) {
		// Internally tracks C while not carrying that on the handler's type
		let erased = ClickHandler::new(
			kinds,
			move |header: *mut Header, click, queue: &mut UiQueue<'_>| {
				let window = header.cast::<WindowData<C>>();
				// SAFETY: the header is the start of this window's WindowData<C>. The trampoline
				// borrows only the header's handlers, so these field borrows are the only others, and its
				// UiGuard excludes every other borrow into the window
				let (layout, data) = unsafe { (&mut (*window).layout, &mut (*window).data) };
				if let Some(layout) = layout {
					handler(layout, data.as_mut(), click, queue);
				}
			},
		);
		// SAFETY: &mut self keeps the window alive
		unsafe { replace(self.ptr, ui, button, Some(erased)) };
	}

	/// Remove `button`'s click handler.
	pub fn clear_click_handler(&mut self, ui: &mut UiGuard, button: Button) {
		// SAFETY: &mut self keeps the window alive
		unsafe { replace(self.ptr, ui, button, None) };
	}
}

/// # Safety
/// `window` must be a live window created by [`Window::new`], with no borrow into it live.
unsafe fn replace(
	window: *mut sys::Window,
	ui: &mut UiGuard,
	button: Button,
	handler: Option<ClickHandler>,
) {
	// SAFETY: forwarded. The provider installed is click_config, for this window
	unsafe {
		replace_with(window, ui, button, handler, || {
			sys::window_set_click_config_provider(window, Some(click_config));
		});
	}
}

/// [`replace`], with `install` installing the click config provider, which the firmware reruns
/// immediately if the window is on screen.
///
/// # Safety
/// As [`replace`], and `install` must only install [`click_config`] for `window`.
pub(super) unsafe fn replace_with(
	window: *mut sys::Window,
	ui: &mut UiGuard,
	button: Button,
	handler: Option<ClickHandler>,
	install: impl FnOnce(),
) {
	// SAFETY: no other borrow into the window is live, and holding a UiGuard excludes a running
	// handler whose box this could drop
	*unsafe { &mut (*header(window)).clicks }.get_mut(button) = handler;
	// SAFETY: user code only gets a `&mut UiGuard` outside callbacks, where nothing borrows a
	// window except through the guard, and the provider only borrows the header's handlers
	unsafe { ui.os_lend(install) };
}

pub(super) unsafe extern "C" fn click_config(context: *mut ffi::c_void) {
	let _ui = UiGuard::enter_internal();
	// SAFETY: we never set a click context, so the firmware passes the window
	let clicks = unsafe { &mut (*header(context.cast())).clicks };
	let mut back_overridden = clicks.back_overridden;
	for button in Button::ALL {
		let Some(handler) = clicks.get_mut(button) else {
			continue;
		};
		let kinds = &handler.subscribed;
		let id = button as sys::ButtonId;
		if matches!(button, Button::Back)
			&& kinds.intersects(ClickKinds::SINGLE | ClickKinds::MULTI)
		{
			back_overridden = true;
		}
		let slot =
			|kind: ClickKinds, f: sys::ClickHandler| kinds.contains(kind).then_some(f).flatten();
		// SAFETY: subscriptions are only valid inside the click config provider, which is here
		unsafe {
			if kinds.contains(ClickKinds::SINGLE) {
				sys::window_single_click_subscribe(
					id,
					Some(click::<{ ClickKinds::SINGLE.bits() }>),
				);
			}
			if kinds.contains(ClickKinds::MULTI) {
				sys::window_multi_click_subscribe(
					id,
					2,
					0,
					0,
					false,
					Some(click::<{ ClickKinds::MULTI.bits() }>),
				);
			}
			if kinds.intersects(ClickKinds::LONG | ClickKinds::LONG_RELEASE) {
				sys::window_long_click_subscribe(
					id,
					0,
					slot(ClickKinds::LONG, Some(click::<{ ClickKinds::LONG.bits() }>)),
					slot(
						ClickKinds::LONG_RELEASE,
						Some(click::<{ ClickKinds::LONG_RELEASE.bits() }>),
					),
				);
			}
			if kinds.intersects(ClickKinds::RAW_DOWN | ClickKinds::RAW_UP) {
				sys::window_raw_click_subscribe(
					id,
					slot(
						ClickKinds::RAW_DOWN,
						Some(click::<{ ClickKinds::RAW_DOWN.bits() }>),
					),
					slot(
						ClickKinds::RAW_UP,
						Some(click::<{ ClickKinds::RAW_UP.bits() }>),
					),
					core::ptr::null_mut(),
				);
			}
		}
	}
	clicks.back_overridden = back_overridden;
	if back_overridden && clicks.back.is_none() {
		// SAFETY: subscriptions are only valid inside the click config provider, which is here
		unsafe {
			sys::window_single_click_subscribe(sys::ButtonId_BUTTON_ID_BACK, Some(back_pop));
		}
	}
}

/// Pops the window, as the firmware's own Back handling does until Back is first subscribed.
unsafe extern "C" fn back_pop(_recognizer: sys::ClickRecognizerRef, _context: *mut ffi::c_void) {
	// SAFETY: popping fires this window's callbacks synchronously, so the guard is lent for it.
	// Nothing here borrows the window
	unsafe { UiGuard::enter_internal().os_lend(|| sys::window_stack_pop(true)) };
}

unsafe extern "C" fn click<const KIND: u8>(
	recognizer: sys::ClickRecognizerRef,
	context: *mut ffi::c_void,
) {
	let mut ui = UiGuard::enter_internal();
	// SAFETY: recognizer is valid for the duration of the handler
	let id = unsafe { sys::click_recognizer_get_button_id(recognizer) };
	let Some(button) = Button::from_raw(id) else {
		return;
	};
	// SAFETY: we never set a click context, so the firmware passes the window. Only the header's
	// handlers are borrowed here; the handler reaches the rest through the raw pointer
	let header = unsafe { header(context.cast()) };
	let Some(handler) = unsafe { &mut (*header).clicks }.get_mut(button) else {
		return;
	};
	let kind = ClickKinds::from_bits_retain(KIND);
	// SAFETY: recognizer is valid for the duration of the handler
	let click = unsafe {
		if kind.contains(ClickKinds::SINGLE) {
			Click::Single {
				repeating: sys::click_recognizer_is_repeating(recognizer),
			}
		} else if kind.contains(ClickKinds::MULTI) {
			Click::Multi {
				count: sys::click_number_of_clicks_counted(recognizer),
			}
		} else if kind.contains(ClickKinds::LONG) {
			Click::Long
		} else if kind.contains(ClickKinds::LONG_RELEASE) {
			Click::LongRelease
		} else if kind.contains(ClickKinds::RAW_DOWN) {
			Click::RawDown
		} else {
			Click::RawUp
		}
	};
	let mut queue = UiQueue::new(&mut ui);
	handler.fun.call((header, click, &mut queue));
}
