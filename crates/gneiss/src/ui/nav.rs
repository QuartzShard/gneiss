//! Deferred window stack navigation.
extern crate alloc;
use alloc::vec::Vec;

use crate::{
	alloc::PebbleAlloc,
	sys,
	ui::{AnyWindow, Layout, TypedWindow, UiGuard, window::WindowData},
};

/// A deferred window stack operation. The `bool` is `animated`.
#[derive(Clone, Copy)]
pub enum NavAction<'n> {
	Push(AnyWindow<'n>, bool),
	Pop(bool),
	PopAll(bool),
	Remove(AnyWindow<'n>, bool),
}

/// Navigation queue. Actions run in the order they were queued when the `Nav` is dropped, which in
/// a window callback is once the handler has returned. Handlers fired by an action run their own
/// queue at that point, before the next action.
pub struct Nav<'n> {
	actions: Vec<NavAction<'n>, PebbleAlloc>,
	ui:      &'n mut UiGuard,
}

impl<'n> Nav<'n> {
	/// An empty queue. It holds `ui` until it drops and runs its actions.
	pub fn new(ui: &'n mut UiGuard) -> Self {
		Self {
			actions: Vec::new_in(PebbleAlloc),
			ui,
		}
	}

	/// Push `window` onto the window stack.
	pub fn push(&mut self, window: impl Into<AnyWindow<'n>>, animated: bool) {
		self.actions.push(NavAction::Push(window.into(), animated));
	}

	/// Push `window`, handing `arg` to its next [`appear`](Layout::appear): straight after this push,
	/// or later if the push doesn't bring it on screen. A newer `push_with` replaces an `arg` that
	/// hasn't been handed over yet.
	pub fn push_with<W: TypedWindow<'n>>(
		&mut self,
		window: W,
		arg: <W::Layout as Layout<W::Data>>::Arg,
		animated: bool,
	) {
		let window: AnyWindow<'n> = window.into();
		// SAFETY: a TypedWindow is a live gneiss window of these types, so its user data is this
		// WindowData. Only `pending` is touched, through a raw pointer: a running handler holds &mut
		// to other fields only, and `pending` is read solely by the appear trampoline, which can't
		// run until this queue drains
		unsafe {
			let data =
				sys::window_get_user_data(window.ptr).cast::<WindowData<W::Layout, W::Data>>();
			drop((&raw mut (*data).pending).replace(Some(arg)));
		}
		self.actions.push(NavAction::Push(window, animated));
	}

	/// Pop whichever window is on top of the stack when this action runs.
	pub fn pop(&mut self, animated: bool) {
		self.actions.push(NavAction::Pop(animated));
	}

	/// Pop every window on the stack when this action runs.
	pub fn pop_all(&mut self, animated: bool) {
		self.actions.push(NavAction::PopAll(animated));
	}

	/// Remove `window` from the window stack, wherever it is when this action runs.
	pub fn remove(&mut self, window: impl Into<AnyWindow<'n>>, animated: bool) {
		self.actions
			.push(NavAction::Remove(window.into(), animated));
	}
}

impl Drop for Nav<'_> {
	fn drop(&mut self) {
		let actions = &mut self.actions;
		self.ui.os_lend(|| {
			for action in actions.drain(..) {
				// SAFETY: every AnyWindow borrows a Window that outlives 'n, and no layout can own a
				// Window (NoWindow), so no earlier action can have destroyed a later target
				unsafe {
					match action {
						NavAction::Push(window, animated) => {
							sys::window_stack_push(window.ptr, animated);
						}
						NavAction::Pop(animated) => {
							sys::window_stack_pop(animated);
						}
						NavAction::PopAll(animated) => sys::window_stack_pop_all(animated),
						NavAction::Remove(window, animated) => {
							sys::window_stack_remove(window.ptr, animated);
						}
					}
				}
			}
		});
	}
}
