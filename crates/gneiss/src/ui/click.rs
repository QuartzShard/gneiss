//! Per-button click handlers for a window.
extern crate alloc;
use alloc::boxed::Box;
use core::ffi;

use crate::{
	alloc::PebbleAlloc,
	sys,
	ui::{Layout, Nav, NoWindow, UiGuard, Window, window::WindowData},
};

/// A window's click handler for each button.
pub struct ClickHandlers<C, D> {
	up:     Option<ClickHandler<C, D>>,
	select: Option<ClickHandler<C, D>>,
	down:   Option<ClickHandler<C, D>>,
	back:   Option<ClickHandler<C, D>>,
}

impl<C, D> ClickHandlers<C, D> {
	pub(super) const fn new() -> Self {
		Self {
			up:     None,
			select: None,
			down:   None,
			back:   None,
		}
	}

	fn get_mut(&mut self, button: Button) -> &mut Option<ClickHandler<C, D>> {
		match button {
			Button::Back => &mut self.back,
			Button::Up => &mut self.up,
			Button::Select => &mut self.select,
			Button::Down => &mut self.down,
		}
	}
}

/// A watch button.
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

	fn from_raw(id: sys::ButtonId) -> Option<Self> {
		Self::ALL.into_iter().find(|b| *b as sys::ButtonId == id)
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
	Single { repeating: bool },
	Multi { count: u8 },
	Long,
	LongRelease,
	RawDown,
	RawUp,
}

/// One button's handler, and the clicks it is subscribed to.
pub struct ClickHandler<C, D> {
	subscribed: ClickKinds,
	fun: Box<
		dyn for<'n> FnMut(&'n mut C, Option<&'n mut D>, Click, &mut Nav<'n>) + NoWindow,
		PebbleAlloc,
	>,
}

impl<C: Layout<D>, D> Window<C, D> {
	/// Replace `button`'s click handler, subscribed to `kinds`. Takes effect immediately if the
	/// window is on screen. Takes effect immediately if the
	/// window is on screen.
	pub fn set_click_handler(
		&mut self,
		ui: &mut UiGuard,
		button: Button,
		kinds: ClickKinds,
		handler: impl for<'n> FnMut(&'n mut C, Option<&'n mut D>, Click, &mut Nav<'n>)
		+ NoWindow
		+ 'static,
	) {
		self.replace_click_handler(
			ui,
			button,
			Some(ClickHandler {
				subscribed: kinds,
				// no alloc for fn items, ZST
				fun:        Box::new_in(handler, PebbleAlloc),
			}),
		);
	}

	/// Remove `button`'s click handler.
	pub fn clear_click_handler(&mut self, ui: &mut UiGuard, button: Button) {
		self.replace_click_handler(ui, button, None);
	}

	fn replace_click_handler(
		&mut self,
		ui: &mut UiGuard,
		button: Button,
		handler: Option<ClickHandler<C, D>>,
	) {
		// SAFETY: &mut self excludes any other borrow of the WindowData, and holding a UiGuard
		// excludes a running handler whose box this could drop
		*unsafe { Self::window_data(self.ptr) }
			.click_handlers
			.get_mut(button) = handler;
		let ptr = self.ptr;
		// SAFETY: ptr is valid. Reruns the provider immediately if the window is on screen
		ui.os_lend(|| unsafe {
			sys::window_set_click_config_provider(ptr, Some(Self::__click_config));
		});
	}
}

impl<C: Layout<D>, D> Window<C, D> {
	pub(super) unsafe extern "C" fn __click_config(context: *mut ffi::c_void) {
		let _ui = UiGuard::enter();
		// SAFETY: we never set a click context, so the firmware passes the window
		let WindowData { click_handlers, .. } = unsafe { Self::window_data(context.cast()) };
		for button in Button::ALL {
			let Some(handler) = click_handlers.get_mut(button) else {
				continue;
			};
			let kinds = &handler.subscribed;
			let id = button as sys::ButtonId;
			let slot = |kind: ClickKinds, f: sys::ClickHandler| {
				kinds.contains(kind).then_some(f).flatten()
			};
			// SAFETY: subscriptions are only valid inside the click config provider, which is here
			unsafe {
				if kinds.contains(ClickKinds::SINGLE) {
					sys::window_single_click_subscribe(
						id,
						Some(Self::__click::<{ ClickKinds::SINGLE.bits() }>),
					);
				}
				if kinds.contains(ClickKinds::MULTI) {
					sys::window_multi_click_subscribe(
						id,
						2,
						0,
						0,
						false,
						Some(Self::__click::<{ ClickKinds::MULTI.bits() }>),
					);
				}
				if kinds.intersects(ClickKinds::LONG | ClickKinds::LONG_RELEASE) {
					sys::window_long_click_subscribe(
						id,
						0,
						slot(
							ClickKinds::LONG,
							Some(Self::__click::<{ ClickKinds::LONG.bits() }>),
						),
						slot(
							ClickKinds::LONG_RELEASE,
							Some(Self::__click::<{ ClickKinds::LONG_RELEASE.bits() }>),
						),
					);
				}
				if kinds.intersects(ClickKinds::RAW_DOWN | ClickKinds::RAW_UP) {
					sys::window_raw_click_subscribe(
						id,
						slot(
							ClickKinds::RAW_DOWN,
							Some(Self::__click::<{ ClickKinds::RAW_DOWN.bits() }>),
						),
						slot(
							ClickKinds::RAW_UP,
							Some(Self::__click::<{ ClickKinds::RAW_UP.bits() }>),
						),
						core::ptr::null_mut(),
					);
				}
			}
		}
	}

	unsafe extern "C" fn __click<const KIND: u8>(
		recognizer: sys::ClickRecognizerRef,
		context: *mut ffi::c_void,
	) {
		let mut ui = UiGuard::enter();
		// SAFETY: recognizer is valid for the duration of the handler
		let id = unsafe { sys::click_recognizer_get_button_id(recognizer) };
		let Some(button) = Button::from_raw(id) else {
			return;
		};
		// SAFETY: we never set a click context, so the firmware passes the window
		let WindowData {
			layout,
			data,
			click_handlers,
			..
		} = unsafe { Self::window_data(context.cast()) };
		let (Some(layout), Some(handler)) = (layout, click_handlers.get_mut(button)) else {
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
		let mut nav = Nav::new(&mut ui);
		(handler.fun)(layout, data.as_mut(), click, &mut nav);
	}
}
