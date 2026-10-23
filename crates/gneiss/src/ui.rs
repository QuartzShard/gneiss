//! Windows, layers, navigation, buttons and drawing.
//!
//! # Windows and layouts
//!
//! The screen shows the top window of a stack. A [`Window<C>`](Window) owns two things:
//!
//! - `C`, its [`Layout`]: the layers and any state needed only while the window is loaded. The
//!   firmware loads a window when it's pushed and unloads it when it leaves the stack, and gneiss
//!   builds `C` in [`Layout::load`] and drops it after [`Layout::unload`] to match.
//! - The layout's [`Data`](Layout::Data), optional data that lives as long as the window does.
//!   Layouts that need none leave it as `()`. [`NoLayout<D>`](NoLayout) is a window with data `D`
//!   and nothing to load.
//!
//! [`Layout::appear`] and [`Layout::disappear`] run whenever the window comes on or goes off
//! screen, including when another window is pushed over it or popped off it.
//!
//! ```no_run
//! # use gneiss::ui::*;
//! struct Counter {
//!     text: TextLayer<HeapText>,
//!     count: u32,
//! }
//!
//! impl Layout for Counter {
//!     fn load(ctx: &mut WindowCtx<'_>) -> Self {
//!         let mut root = ctx.root();
//!         let mut text = TextLayer::new(root.bounds()).expect("text layer");
//!         text.set_text("0");
//!         root.add_child(&mut text);
//!         Self { text, count: 0 }
//!     }
//! }
//! ```
//!
//! # Layers
//!
//! A layout owns its layers, and each layer owns what it shows: a [`TextLayer`] its text and font,
//! a [`BitmapLayer`] its bitmap, and a [`DataLayer<T>`](DataLayer) a `T` that your [`DrawFn`]
//! draws. Dropping the layout drops them all, and nothing the firmware still points at is freed
//! first. Layer setters return `&mut Self`, so they chain.
//!
//! A layer you never touch again after `load` still has to live in a field, so the layout keeps it
//! alive. Name such fields with a leading underscore (`_logo`) so Rust doesn't warn that they're
//! never read.
//!
//! # Where windows live
//!
//! A window that has to outlive `main` lives in a [`StaticWindow`], or inside another window's
//! [`Data`](Layout::Data). A layout can't own a window ([`NoWindow`]): layouts are dropped when their window
//! unloads, which happens partway through navigation.
//!
//! # Buttons
//!
//! [`Window::set_click_handler`] gives one button a closure, subscribed to the [`ClickKinds`] it
//! wants. It runs with the layout, the window's data, the [`Click`] that happened, and a
//! [`UiQueue`]. Back pops the window unless you give it a handler.
//!
//! # Navigation
//!
//! Pushing or popping a window fires callbacks straight away: the old window's `disappear`, maybe
//! its `unload`, then the new one's `load` and `appear`. From inside a callback that would hand out
//! a second `&mut` to a layout that's still in use, so navigation is queued on a [`UiQueue`] and
//! runs once the callback returns. [`UiQueue::push_with`] also hands the pushed window an argument,
//! which its next `appear` receives.
//!
//! # The UI guard
//!
//! [`UiGuard`] is the token behind those rules. At most one exists: gneiss holds it around every
//! UI callback it runs, `main` is given one, and anything else, such as a service handler or a
//! timer, takes its own with [`UiGuard::enter`]. Anything that reaches into a window takes it as
//! an argument.
//!
//! # Updating the screen from a service
//!
//! A service handler reaches a window through its [`StaticWindow`]: [`StaticWindow::with`] lends
//! the layout, if the window is loaded, and the window's data. [`StatusBoard`] is a ready-made
//! window for showing a few lines of status this way.
//!
//! # Drawing
//!
//! [`graphics`] draws with the firmware's own primitives through
//! [`GContext`](graphics::GContext), or with `embedded-graphics` straight into the framebuffer.
use core::ptr::NonNull;

use crate::{
	SyncCell,
	fail::PblUnwrap,
	sys,
	ui::window::header,
};

mod action_bar;
mod action_menu;
mod bitmap;
mod board;
mod click;
pub mod graphics;
mod layer;
mod menu;
mod scroll;
mod simple_menu;
mod status_bar;
mod text;
mod window;

pub use action_bar::{
	ACTION_BAR_WIDTH, ActionBar, ActionButton, ActionClickKinds, ActionWindow, PressAnimation,
};
pub use action_menu::{
	ActionFn, ActionItem, ActionMenu, ActionMenuAlign, ActionMenuStyle, LevelMode, Reply,
	ResultWindow,
};
pub use bitmap::{Align, BitmapLayer, RotBitmapLayer};
pub use board::{BoardLayout, BoardLines, StatusBoard};
pub use click::{Button, Click, ClickHandler, ClickHandlers, ClickKinds};
pub use layer::{AsLayer, DataLayer, DrawFn, Layer, RootLayer};
#[doc(hidden)]
pub use menu::__trampolines as __menu;
pub use menu::{Menu, MenuAlign, MenuCanvas, MenuIndex, MenuRef, MenuWindow};
pub use scroll::{ScrollLayer, ScrollWindow};
pub use simple_menu::{
	MenuItem, MenuList, MenuListFill, MenuSection, Sections, SimpleMenuData, SimpleMenuWindow,
};
pub use status_bar::{STATUS_BAR_HEIGHT, Separator, StatusBarLayer};
pub use text::{HeapText, StackText, TextAlignment, TextLayer};
pub use window::{
	AnyWindow, Layout, LeakWindow, NoLayout, NoWindow, StaticWindow, TypedWindow, Window,
	WindowCtx, WindowRef,
};

static UI_HELD: SyncCell<bool> = SyncCell::new(false);
static UI_LENT: SyncCell<u8> = SyncCell::new(0);

/// No guard is held or lent: no window callback or queue is running.
pub(super) fn ui_idle() -> bool {
	!UI_HELD.get() && UI_LENT.get() == 0
}

/// Permission to touch the UI: borrowing a window's layout or data, changing click handlers,
/// navigating, reloading a menu. At most one is usable at a time. gneiss holds it around every UI
/// callback it runs (window lifecycle, layer drawing, clicks, every menu callback),
/// `#[gneiss::entry]` hands one to `main`, and anything else, such as a service handler or timer,
/// takes its own with [`enter`](Self::enter). While gneiss calls into the firmware from a callback,
/// it lends its guard, so callbacks the firmware fires synchronously can take their own; the lent
/// guard can't be used until they return. So a `&mut UiGuard` proves no other code holds a borrow
/// into any window, and a shared `&UiGuard` proves none is held mutably.
///
/// Some window-related actions can cause reentrant firing of handlers if they're caused from within
/// one. This causes `&mut` aliasing UB if it happens, so this type is used as a proof of "good
/// behaviour", panicking instead of permitting the UB to occur if reentrance is triggered:
/// ```text
/// _appear(win_a)
///   let a1 = &mut A ───────────────────────┐ 'a1
///   A::appear(a1)                          │
///     window_stack_pop()                   │
///       _disappear(win_a)                  │
///         let a2 = &mut A ────────┐ 'a2    │  'a2 inside 'a1: two live &mut A, UB.
///         A::disappear(a2)        │        │  UiGuard::enter turns this into a panic
///       ◄─────────────────────────┘        │
///       _unload(win_a)                     │
///         drop(A)                          │  a1 now dangles
///     a1.foo = 1  ◄────────────────────────┘  write to freed A
/// ```
/// Actions that can cause the firing of handlers, or would be rendered unsound by it happening, take
/// this guard as a proof token.
///
/// Holding the token does not put you at risk of panic when an event happens: there's no
/// preemption. Only reentrance will trigger it.
///
/// A [`Window`] dropped while a guard is held is destroyed once the guard is released.
///
/// Forgetting or otherwise leaking a guard makes every later window callback panic, and keeps every
/// [`Window`] dropped afterwards from being destroyed. Don't do that
pub struct UiGuard(());

impl UiGuard {
	/// Take the guard.
	///
	/// # Panics
	/// If a guard is already held, e.g. from inside a window callback.
	#[track_caller]
	pub fn enter() -> Self {
		Self::try_enter().pbl_expect(c"UiGuard already held")
	}

	/// [`enter`](Self::enter) for gneiss's own callbacks, failing at `gneiss:<line>` rather than
	/// recording where it was called.
	pub(crate) fn enter_internal() -> Self {
		crate::fail::gneiss_expect!(Self::try_enter(), "UiGuard already held")
	}

	/// Take the guard. `None` if one is already held, e.g. from inside a window callback.
	pub fn try_enter() -> Option<Self> {
		(!UI_HELD.replace(true)).then_some(Self(()))
	}

	/// Release the guard while `f` calls into the firmware, so window callbacks it fires
	/// synchronously can take their own.
	///
	/// # Safety
	/// The `&mut` borrow only rules out borrows made through the guard. Those callbacks borrow
	/// window state through raw pointers, so no reference into a window that one of them could also
	/// borrow may be used once `f` starts: the caller's must be dead, and `f` must make none.
	#[expect(
		clippy::unused_self,
		reason = "the &mut borrow is the proof of exclusivity"
	)]
	pub(crate) unsafe fn os_lend<R>(&mut self, f: impl FnOnce() -> R) -> R {
		UI_HELD.set(false);
		UI_LENT.set(UI_LENT.get() + 1);
		let result = f();
		UI_LENT.set(UI_LENT.get() - 1);
		crate::fail::gneiss_assert!(
			!UI_HELD.replace(true),
			"UiGuard leaked by a window callback"
		);
		result
	}
}

impl Drop for UiGuard {
	fn drop(&mut self) {
		UI_HELD.set(false);
		// The last guard: any window dropped while it was held can go now
		if UI_LENT.get() == 0 {
			window::destroy_dropped();
		}
	}
}

/// A deferred UI action. The `bool` is `animated`.
#[derive(Clone, Copy)]
pub enum UiAction<'n> {
	Push(AnyWindow<'n>, bool),
	Pop(bool),
	PopAll(bool),
	Remove(AnyWindow<'n>, bool),
	/// Reload a [`MenuWindow`]'s menu.
	Reload(NonNull<sys::MenuLayer>),
	ActionMenu(NonNull<sys::ActionMenuConfig>),
}

/// UI actions deferred until the callback queuing them is done: navigation, and menu reloads.
/// Actions run in the order they were queued when the `UiQueue` is dropped, which in a window
/// callback is once the handler has returned and its borrows have ended. Handlers fired by an
/// action run their own queue at that point, before the next action.
///
/// Holds up to [`UiQueue::CAPACITY`] actions; queuing more logs a warning and drops the new action.
pub struct UiQueue<'n> {
	actions: [Option<UiAction<'n>>; UiQueue::CAPACITY],
	len:     usize,
	ui:      &'n mut UiGuard,
	// The menu whose select callback this queue was made for
	menu:    Option<NonNull<sys::MenuLayer>>,
}

impl<'n> UiQueue<'n> {
	/// How many actions one `UiQueue` can queue.
	pub const CAPACITY: usize = 16;

	/// An empty queue. It holds `ui` until it drops and runs its actions.
	pub fn new(ui: &'n mut UiGuard) -> Self {
		Self {
			actions: [None; Self::CAPACITY],
			len: 0,
			ui,
			menu: None,
		}
	}

	/// A queue for `menu`'s select callback, which [`reload`](Self::reload) reloads.
	pub(super) fn for_menu(ui: &'n mut UiGuard, menu: NonNull<sys::MenuLayer>) -> Self {
		let mut queue = Self::new(ui);
		queue.menu = Some(menu);
		queue
	}

	fn queue(&mut self, action: UiAction<'n>) {
		let Some(slot) = self.actions.get_mut(self.len) else {
			crate::log::gneiss_log!(Warn, "queue full");
			return;
		};
		*slot = Some(action);
		self.len += 1;
	}

	/// Push `window` onto the window stack.
	pub fn push(&mut self, window: impl Into<AnyWindow<'n>>, animated: bool) {
		self.queue(UiAction::Push(window.into(), animated));
	}

	/// Push `window`, handing `arg` to its next [`appear`](Layout::appear): straight after this push,
	/// or later if the push doesn't bring it on screen. A newer `push_with` replaces an `arg` that
	/// hasn't been handed over yet.
	pub fn push_with<W: TypedWindow<'n>>(
		&mut self,
		window: W,
		arg: <W::Layout as Layout>::Arg,
		animated: bool,
	) {
		self.queue(UiAction::Push(window::hand_arg(window, arg), animated));
	}

	/// Pop whichever window is on top of the stack when this action runs.
	pub fn pop(&mut self, animated: bool) {
		self.queue(UiAction::Pop(animated));
	}

	/// Pop whichever window is on top of the stack when this action runs, handing `arg` to
	/// `window`'s next [`appear`](Layout::appear): straight after this pop if `window` is under the
	/// top. As for [`push_with`](Self::push_with), a newer `arg` replaces one not yet handed over.
	pub fn pop_with<W: TypedWindow<'n>>(
		&mut self,
		window: W,
		arg: <W::Layout as Layout>::Arg,
		animated: bool,
	) {
		window::hand_arg(window, arg);
		self.queue(UiAction::Pop(animated));
	}

	/// Pop every window on the stack when this action runs.
	pub fn pop_all(&mut self, animated: bool) {
		self.queue(UiAction::PopAll(animated));
	}

	/// Open an action menu showing `root`. Picking an item runs its [`ActionFn`] on `data`, then the
	/// menu closes onto the [`ResultWindow`] the action returns, if any. `data` drops once the menu has
	/// closed.
	///
	/// Logs a warning instead if out of memory, over `u16::MAX` actions, or the queue is full.
	pub fn action_menu<D: NoWindow + 'static>(
		&mut self,
		root: &ActionMenu<'_, D>,
		data: D,
		style: ActionMenuStyle,
	) {
		self.open_action_menu(root, data, style, None);
	}

	/// Like [`action_menu`](Self::action_menu), but actions can also [`Reply::Arg`] to `target`,
	/// such as the window opening it.
	pub fn action_menu_to<W: TypedWindow<'static>, D: NoWindow + 'static>(
		&mut self,
		target: W,
		root: &ActionMenu<'_, D, <W::Layout as Layout>::Arg>,
		data: D,
		style: ActionMenuStyle,
	) {
		let deliver = window::hand_arg_to::<W::Layout>;
		self.open_action_menu(root, data, style, Some((target.into(), deliver)));
	}

	fn open_action_menu<D: NoWindow + 'static, A: 'static>(
		&mut self,
		root: &ActionMenu<'_, D, A>,
		data: D,
		style: ActionMenuStyle,
		target: Option<action_menu::Target<A>>,
	) {
		if self.len == Self::CAPACITY {
			crate::log::gneiss_log!(Warn, "queue full");
			return;
		}
		match action_menu::build(root, data, style, target) {
			Some(config) => self.queue(UiAction::ActionMenu(config)),
			None => crate::log::gneiss_log!(Warn, "action menu not built"),
		}
	}

	/// Remove `window` from the window stack, wherever it is when this action runs.
	pub fn remove(&mut self, window: impl Into<AnyWindow<'n>>, animated: bool) {
		self.queue(UiAction::Remove(window.into(), animated));
	}

	/// Reload the menu whose select callback this queue was handed to, after changing its data, as
	/// [`MenuRef::reload`](MenuRef::reload) does. Outside a menu's select callback, logs a
	/// warning and does nothing: use [`reload_menu`](Self::reload_menu).
	pub fn reload(&mut self) {
		if let Some(menu) = self.menu {
			self.queue(UiAction::Reload(menu));
		} else {
			crate::log::gneiss_log!(Warn, "reload outside a menu's select callback");
		}
	}

	/// Reload `window`'s menu after changing its data. Does nothing unless `window` is a
	/// [`MenuWindow`].
	pub fn reload_menu(&mut self, window: impl Into<AnyWindow<'n>>) {
		let window = window.into();
		// SAFETY: every AnyWindow is a live gneiss window. Only its menu is read, which nothing
		// writes after the window is made
		if let Some(menu) = unsafe { (*header(window.ptr)).menu } {
			self.queue(UiAction::Reload(menu));
		}
	}
}

impl Drop for UiQueue<'_> {
	fn drop(&mut self) {
		let actions = &self.actions;
		let run = || {
			for &action in actions.iter().flatten() {
				// SAFETY: every AnyWindow borrows a Window that outlives 'n, and no layout can own a
				// Window (NoWindow), so no earlier action can have destroyed a later target
				unsafe {
					match action {
						UiAction::Push(window, animated) => {
							sys::window_stack_push(window.ptr, animated);
						}
						UiAction::Pop(animated) => {
							sys::window_stack_pop(animated);
						}
						UiAction::PopAll(animated) => sys::window_stack_pop_all(animated),
						UiAction::Remove(window, animated) => {
							sys::window_stack_remove(window.ptr, animated);
						}
						// Fires the menu's callbacks, which read its window's data: the handler that
						// queued this has returned, so nothing borrows it
						UiAction::Reload(menu) => sys::menu_layer_reload_data(menu.as_ptr()),
						// Freed when the menu closes
						UiAction::ActionMenu(config) => {
							sys::action_menu_open(config.as_ptr());
						}
					}
				}
			}
		};
		// SAFETY: in a callback, the queue drops once the handler it was handed to has returned, and
		// the trampoline doesn't use the handler's borrows again. Outside one, borrows into windows go
		// through the guard, which the queue holds
		unsafe { self.ui.os_lend(run) };
	}
}
