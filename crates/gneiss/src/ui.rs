//! Windows, layers and navigation, and the [`UiGuard`] token that keeps their callbacks from
//! aliasing.
use core::cell::Cell;

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

struct UiHeld(Cell<bool>);

// SAFETY: PebbleOS runs all app code on a single task.
// Not an atomic: read-modify-write atomics spin forever!
unsafe impl Sync for UiHeld {}

static UI_HELD: UiHeld = UiHeld(Cell::new(false));

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
		(!UI_HELD.0.replace(true)).then_some(Self(()))
	}

	/// Release the guard while `f` calls into the firmware, so window callbacks it fires
	/// synchronously can take their own.
	#[expect(
		clippy::unused_self,
		reason = "the &mut borrow is the proof of exclusivity"
	)]
	pub(crate) fn os_lend<R>(&mut self, f: impl FnOnce() -> R) -> R {
		UI_HELD.0.set(false);
		let result = f();
		assert!(
			!UI_HELD.0.replace(true),
			"UiGuard leaked by a window callback"
		);
		result
	}
}

impl Drop for UiGuard {
	fn drop(&mut self) {
		UI_HELD.0.set(false);
	}
}
