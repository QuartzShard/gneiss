//! Windows, layers, navigation, buttons and drawing.
//!
//! # Windows and layouts
//!
//! The screen shows the top window of a stack. A [`Window<C, D>`](Window) owns two things:
//!
//! - `D`, optional data that lives as long as the window does.
//! - `C`, its [`Layout`]: the layers and any state needed only while the window is loaded. The
//!   firmware loads a window when it's pushed and unloads it when it leaves the stack, and gneiss
//!   builds `C` in [`Layout::load`] and drops it after [`Layout::unload`] to match.
//!
//! [`Layout::appear`] and [`Layout::disappear`] run whenever the window comes on or goes off
//! screen, including when another window is pushed over it or popped off it.
//!
//! ```no_run
//! # use gneiss::ui::*;
//! struct Counter {
//!     text: TextLayer<TextBuf>,
//!     count: u32,
//! }
//!
//! impl Layout for Counter {
//!     fn load(ctx: &mut WindowCtx<'_, Self>) -> Self {
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
//! `D`. A layout can't own a window ([`NoWindow`]): layouts are dropped when their window
//! unloads, which happens partway through navigation.
//!
//! # Buttons
//!
//! [`Window::set_click_handler`] gives one button a closure, subscribed to the [`ClickKinds`] it
//! wants. It runs with the layout, the window's data, the [`Click`] that happened, and a [`Nav`].
//! Back pops the window unless you give it a handler.
//!
//! # Navigation
//!
//! Pushing or popping a window fires callbacks straight away: the old window's `disappear`, maybe
//! its `unload`, then the new one's `load` and `appear`. From inside a callback that would hand out
//! a second `&mut` to a layout that's still in use, so navigation is queued on a [`Nav`] and runs
//! once the callback returns. [`Nav::push_with`] also hands the pushed window an argument, which its
//! next `appear` receives.
//!
//! # The UI guard
//!
//! [`UiGuard`] is the token behind those rules. At most one exists, every window callback holds it
//! while it runs, and anything that reaches into a window takes it as an argument. `main` is given
//! one; a service handler takes its own with [`UiGuard::enter`].
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
use crate::cell::SyncCell;

mod bitmap;
mod board;
mod click;
pub mod graphics;
mod layer;
mod nav;
mod text;
mod window;

pub use bitmap::{Align, BitmapLayer};
pub use board::{BoardLayout, BoardLines, StatusBoard};
pub use click::{Button, Click, ClickHandler, ClickHandlers, ClickKinds};
pub use layer::{AsLayer, DataLayer, DrawFn, Layer, RootLayer};
pub use nav::{Nav, NavAction};
pub use text::{TextAlignment, TextBuf, TextLayer};
pub use window::{
	AnyWindow, Layout, NoWindow, StaticWindow, TypedWindow, Window, WindowCtx, WindowRef,
};

static UI_HELD: SyncCell<bool> = SyncCell::new(false);

/// Permission to touch the UI: borrowing a layout, changing click handlers, navigating. At most one
/// exists at a time, and every window callback holds one while it runs, so a `&mut UiGuard` proves
/// no other code holds a borrow into any window, and a shared `&UiGuard` proves none is held
/// mutably.
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
/// Forgetting or otherwise leaking a guard makes every later window callback panic.
pub struct UiGuard(());

impl UiGuard {
	/// Take the guard.
	///
	/// # Panics
	/// If a guard is already held, e.g. from inside a window callback.
	pub fn enter() -> Self {
		Self::try_enter().expect("UiGuard already held")
	}

	/// Take the guard. `None` if one is already held, e.g. from inside a window callback.
	pub fn try_enter() -> Option<Self> {
		(!UI_HELD.replace(true)).then_some(Self(()))
	}

	/// Release the guard while `f` calls into the firmware, so window callbacks it fires
	/// synchronously can take their own.
	#[expect(
		clippy::unused_self,
		reason = "the &mut borrow is the proof of exclusivity"
	)]
	pub(crate) fn os_lend<R>(&mut self, f: impl FnOnce() -> R) -> R {
		UI_HELD.set(false);
		let result = f();
		assert!(
			!UI_HELD.replace(true),
			"UiGuard leaked by a window callback"
		);
		result
	}
}

impl Drop for UiGuard {
	fn drop(&mut self) {
		UI_HELD.set(false);
	}
}
