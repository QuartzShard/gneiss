//! Action menus are the lists that slide in from the right, such as a notification's Dismiss and
//! Reply. Open one with [`UiQueue::action_menu`](super::UiQueue::action_menu).
extern crate alloc;
use alloc::boxed::Box;
use core::{
	convert::Infallible,
	ffi::{CStr, c_void},
	ptr::{self, NonNull},
};

use crate::{
	alloc::{Arena, ArenaFill, Handle},
	sys,
	ui::{AnyWindow, Layout, NoWindow, TypedWindow, UiGuard, graphics::Colour, window},
};

/// Runs when its item is picked, with the menu's data and the item's index in its level's
/// [`items`](ActionMenu::items). The guard reaches other windows through
/// [`StaticWindow::borrow`](super::StaticWindow::borrow).
pub type ActionFn<D, A = Infallible> = fn(&mut D, u16, &UiGuard) -> Option<Reply<A>>;

/// What an action hands back as the menu closes: a window to close onto, or an argument for the
/// window the menu was opened for with [`UiQueue::action_menu_to`](super::UiQueue::action_menu_to),
/// which its next [`appear`](Layout::appear) receives.
pub enum Reply<A = Infallible> {
	Window(ResultWindow),
	Arg(A),
}

/// A window for the menu to close onto, such as a confirmation, with the argument its next
/// [`appear`](Layout::appear) receives, as [`UiQueue::push_with`](super::UiQueue::push_with) hands
/// one.
pub struct ResultWindow(AnyWindow<'static>);

impl ResultWindow {
	pub fn new<W: TypedWindow<'static>>(window: W, arg: <W::Layout as Layout>::Arg) -> Self {
		Self(window::hand_arg(window, arg))
	}
}

/// A label, and what picking it does.
pub enum ActionItem<'t, D, A = Infallible> {
	Action(&'t CStr, ActionFn<D, A>),
	Level(&'t CStr, &'t ActionMenu<'t, D, A>),
}

/// The root of an action menu, or one of its submenus.
/// [`UiQueue::action_menu`](super::UiQueue::action_menu) copies the whole tree, so labels and
/// levels can borrow locals.
///
/// Each label and function in a `static` tree needs a relocation at load, so building a big tree in
/// locals keeps the app smaller.
///
/// ```no_run
/// # use gneiss::ui::*;
/// struct Alarm {
///     enabled: bool,
/// }
///
/// fn toggle(alarm: &mut Alarm, _: u16, _: &UiGuard) -> Option<Reply> {
///     alarm.enabled = !alarm.enabled;
///     None
/// }
///
/// const SNOOZE_MINUTES: [u8; 3] = [5, 10, 15];
///
/// fn snooze(_: &mut Alarm, index: u16, _: &UiGuard) -> Option<Reply> {
///     let _minutes = SNOOZE_MINUTES[usize::from(index)];
///     None
/// }
///
/// fn open(alarm: Alarm, queue: &mut UiQueue<'_>) {
///     let snooze_level = ActionMenu::new(&[
///         ActionItem::Action(c"5 minutes", snooze),
///         ActionItem::Action(c"10 minutes", snooze),
///         ActionItem::Action(c"15 minutes", snooze),
///     ]);
///     let label = if alarm.enabled { c"Disable" } else { c"Enable" };
///     let items = [
///         ActionItem::Action(label, toggle),
///         ActionItem::Level(c"Snooze", &snooze_level),
///     ];
///     queue.action_menu(&ActionMenu::new(&items), alarm, ActionMenuStyle::default());
/// }
/// ```
pub struct ActionMenu<'t, D, A = Infallible> {
	pub mode:  LevelMode,
	pub items: &'t [ActionItem<'t, D, A>],
}

impl<'t, D, A> ActionMenu<'t, D, A> {
	/// Shows `items` in the [`Wide`](LevelMode::Wide) layout.
	pub const fn new(items: &'t [ActionItem<'t, D, A>]) -> Self {
		Self {
			mode: LevelMode::Wide,
			items,
		}
	}
}

/// `Wide` gives each item its own row and wraps long labels. `Thin` puts short items side by side,
/// such as emoji.
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum LevelMode {
	#[default]
	Wide = sys::ActionMenuLevelDisplayMode_ActionMenuLevelDisplayModeWide,
	Thin = sys::ActionMenuLevelDisplayMode_ActionMenuLevelDisplayModeThin,
}

/// Where the selected item sits on screen.
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum ActionMenuAlign {
	#[default]
	Top    = sys::ActionMenuAlign_ActionMenuAlignTop,
	Center = sys::ActionMenuAlign_ActionMenuAlignCenter,
}

/// Defaults to the firmware's look.
/// Colours ignored on B&W
#[derive(Clone, Copy, Default)]
pub struct ActionMenuStyle {
	/// The strip down the left.
	pub background: Option<Colour>,
	/// The dots on the strip, one per level. `None` picks one legible over `background`.
	pub crumbs:     Option<Colour>,
	pub align:      ActionMenuAlign,
}

/// Where an action's [`Reply::Arg`] goes.
pub(super) type Target<A> = (AnyWindow<'static>, unsafe fn(AnyWindow<'static>, A));

struct Open<D, A> {
	config:   sys::ActionMenuConfig,
	arena:    Arena,
	handlers: Handle<[usize]>,
	target:   Option<Target<A>>,
	data:     D,
}

/// Builds the config for [`sys::action_menu_open`]. Returns `None` if out of memory, or if the tree
/// has more than `u16::MAX` actions.
pub(super) fn build<D: NoWindow + 'static, A: 'static>(
	root: &ActionMenu<'_, D, A>,
	data: D,
	style: ActionMenuStyle,
	target: Option<Target<A>>,
) -> Option<NonNull<sys::ActionMenuConfig>> {
	let mut measure = ArenaFill::measuring();
	let mut actions = 0;
	count(root, &mut measure, &mut actions);
	let actions = u16::try_from(actions).ok()?;
	let capacity = size_of::<usize>()
		.checked_mul(usize::from(actions))?
		.checked_add(measure.used())?;
	let mut fill = Arena::new_with_capacity(capacity)?;
	let handlers = fill.reserve_slice::<usize>(usize::from(actions));
	let mut slot = 0;
	let root_level = create::<D, A>(root, &mut fill, handlers, &mut slot)?;
	let Some(arena) = fill.finish() else {
		// SAFETY: created above, and nothing else points at it
		unsafe { sys::action_menu_hierarchy_destroy(root_level, None, ptr::null_mut()) };
		return None;
	};
	let open = Box::into_raw(Box::new(Open {
		config: sys::ActionMenuConfig {
			root_level,
			context: ptr::null_mut(),
			colors: sys::ActionMenuConfig__bindgen_ty_1 {
				background: style.background.unwrap_or(Colour { argb: 0 }).into(),
				foreground: style.crumbs.unwrap_or(Colour { argb: 0 }).into(),
			},
			will_close: None,
			did_close: Some(did_close::<D, A>),
			align: style.align as sys::ActionMenuAlign,
		},
		arena,
		handlers,
		target,
		data,
	}));
	// SAFETY: just allocated
	unsafe {
		(*open).config.context = open.cast();
		Some(NonNull::new_unchecked(&raw mut (*open).config))
	}
}

// Counts actions and measures labels
fn count<D, A>(level: &ActionMenu<'_, D, A>, fill: &mut ArenaFill, actions: &mut usize) {
	for item in level.items {
		match item {
			ActionItem::Action(label, _) => {
				fill.push_c_str(label);
				*actions += 1;
			}
			ActionItem::Level(label, child) => {
				fill.push_c_str(label);
				count(child, fill, actions);
			}
		}
	}
}

// On out of memory, frees what it made and returns `None`
fn create<D: 'static, A: 'static>(
	level: &ActionMenu<'_, D, A>,
	fill: &mut ArenaFill,
	handlers: Handle<[usize]>,
	slot: &mut usize,
) -> Option<*mut sys::ActionMenuLevel> {
	let len = u16::try_from(level.items.len()).ok()?;
	// SAFETY: no preconditions; NULL on OOM
	let made = unsafe { sys::action_menu_level_create(len).as_mut()? };
	// SAFETY: just created
	unsafe { sys::action_menu_level_set_display_mode(made, level.mode as _) };
	for (index, item) in (0..len).zip(level.items) {
		match item {
			ActionItem::Action(label, handler) => {
				let label = fill.push_c_str(label);
				#[expect(
					clippy::cast_ptr_alignment,
					reason = "reserve_slice aligned it for usize"
				)]
				let at = fill.ptr(handlers).cast_mut().cast::<usize>();
				// SAFETY: count() reserved a slot for every action, and create() visits them in the
				// same order
				unsafe { at.add(*slot).write(*handler as usize) };
				let packed = *slot | usize::from(index) << 16;
				*slot += 1;
				// SAFETY: made has room for len items, and the label lives in the arena, which the
				// menu frees after the levels
				unsafe {
					sys::action_menu_level_add_action(
						made,
						fill.ptr(label).cast(),
						Some(perform::<D, A>),
						ptr::without_provenance_mut(packed),
					);
				}
			}
			ActionItem::Level(label, child) => {
				let label = fill.push_c_str(label);
				let Some(child) = create(child, fill, handlers, slot) else {
					// SAFETY: made and the levels added to it are ours alone
					unsafe { sys::action_menu_hierarchy_destroy(made, None, ptr::null_mut()) };
					return None;
				};
				// SAFETY: as for an action; made now owns child
				unsafe { sys::action_menu_level_add_child(made, child, fill.ptr(label).cast()) };
			}
		}
	}
	Some(made)
}

unsafe extern "C" fn perform<D: 'static, A: 'static>(
	menu: *mut sys::ActionMenu,
	item: *const sys::ActionMenuItem,
	context: *mut c_void,
) {
	let open = context.cast::<Open<D, A>>();
	// SAFETY: every action's data is the packed slot and index create() gave it
	let packed = unsafe { sys::action_menu_item_get_action_data(item) }.addr();
	let (slot, index) = (packed & 0xFFFF, (packed >> 16) as u16);
	// SAFETY: context is the Open the menu was built with, alive until did_close. create() wrote an
	// ActionFn<D> into each slot an action points at
	let handler = unsafe {
		let raw = *(*open)
			.arena
			.slice_unchecked((*open).handlers)
			.get_unchecked(slot);
		core::mem::transmute::<usize, ActionFn<D, A>>(raw)
	};
	// Held so the action can't navigate: the firmware still uses the menu once this returns
	let ui = UiGuard::enter_internal();
	// SAFETY: the firmware runs one action at a time, and nothing else borrows the data
	let reply = handler(unsafe { &mut (*open).data }, index, &ui);
	drop(ui);
	match reply {
		// SAFETY: the menu is open, and the window is 'static. Inserting it fires its callbacks,
		// which take their own guard
		Some(Reply::Window(window)) => unsafe {
			sys::action_menu_set_result_window(menu, window.0.ptr);
		},
		Some(Reply::Arg(arg)) => {
			// SAFETY: action_menu_to paired the window with hand_arg_to for its layout
			if let Some((window, deliver)) = unsafe { (*open).target } {
				unsafe { deliver(window, arg) };
			}
		}
		None => {}
	}
}

unsafe extern "C" fn did_close<D: 'static, A: 'static>(
	_menu: *mut sys::ActionMenu,
	_performed: *const sys::ActionMenuItem,
	context: *mut c_void,
) {
	// SAFETY: context is the Open build() leaked, and the menu is done with it
	let open = unsafe { Box::from_raw(context.cast::<Open<D, A>>()) };
	// SAFETY: the levels are the menu's, which is closing
	unsafe { sys::action_menu_hierarchy_destroy(open.config.root_level, None, ptr::null_mut()) };
}
