//! [`SimpleMenuWindow`], a window showing a fixed list of rows, drawn and counted by the firmware.
use core::{
	ffi::{CStr, c_int, c_void},
	marker::PhantomData,
	ptr::{self, NonNull},
};

pub use list::{MenuList, MenuListFill};

use crate::{
	resource::image::GBitMap,
	sys,
	ui::{
		AnyWindow, MenuIndex, MenuRef, NoLayout, TypedWindow, UiGuard, UiQueue, Window, WindowCtx,
		window::{LeakWindow, header},
	},
};

mod list;

/// One row of a [`SimpleMenuWindow`]: a title, and an optional subtitle under it.
#[repr(transparent)]
pub struct MenuItem<'a> {
	raw:       sys::SimpleMenuItem,
	_title:    PhantomData<&'a CStr>,
	_subtitle: PhantomData<Option<&'a CStr>>,
	_icon:     PhantomData<Option<&'a GBitMap>>,
}

impl<'a> MenuItem<'a> {
	/// A row showing `title`.
	pub const fn new(title: &'a CStr) -> Self {
		Self {
			raw:       sys::SimpleMenuItem {
				title:    title.as_ptr(),
				subtitle: ptr::null(),
				icon:     ptr::null_mut(),
				callback: Some(simple_menu_select),
			},
			_title:    PhantomData,
			_subtitle: PhantomData,
			_icon:     PhantomData,
		}
	}

	/// Show `subtitle` under the title.
	#[must_use]
	pub const fn subtitle(mut self, subtitle: &'a CStr) -> Self {
		self.raw.subtitle = subtitle.as_ptr();
		self
	}

	/// Show `icon` left of the title. Bitmaps are loaded at runtime, so a row with an icon can't be
	/// in a `static` table.
	#[must_use]
	pub fn icon(mut self, icon: &'a GBitMap) -> Self {
		self.raw.icon = icon.as_ptr();
		self
	}
}

/// A group of rows in a [`SimpleMenuWindow`], with an optional header.
#[repr(transparent)]
pub struct MenuSection<'a> {
	raw:    sys::SimpleMenuSection,
	_items: PhantomData<&'a [MenuItem<'a>]>,
}

impl<'a> MenuSection<'a> {
	/// A section of `items`, with no header.
	pub const fn new(items: &'a [MenuItem<'a>]) -> Self {
		Self {
			raw:    sys::SimpleMenuSection {
				title:     ptr::null(),
				items:     items.as_ptr().cast(),
				num_items: items.len() as u32,
			},
			_items: PhantomData,
		}
	}

	/// Give the section a header showing `title`.
	#[must_use]
	pub const fn title(mut self, title: &'a CStr) -> Self {
		self.raw.title = title.as_ptr();
		self
	}
}

// SAFETY: the pointers only reach immutable strings and items borrowed for 'a
unsafe impl Sync for MenuItem<'_> {}
// SAFETY: as for MenuItem
unsafe impl Sync for MenuSection<'_> {}

mod sealed {
	pub trait Sealed {}
}

/// What a [`SimpleMenuWindow`] shows: a `static` table of [`MenuSection`]s, or a [`MenuList`]
/// built at runtime.
pub trait Sections: sealed::Sealed {
	#[doc(hidden)]
	fn sections(&self) -> &[MenuSection<'_>];
}

impl<T: AsRef<[MenuSection<'static>]>> sealed::Sealed for T {}

impl<T: AsRef<[MenuSection<'static>]>> Sections for T {
	fn sections(&self) -> &[MenuSection<'_>] {
		self.as_ref()
	}
}

/// What every item's callback reads through the firmware's context pointer: kept first in
/// [`SimpleMenuData`].
#[repr(C)]
struct SimpleMenuHeader {
	// Typed dispatch for this window's data
	dispatch: unsafe fn(*mut c_void, MenuIndex),
	menu:     *mut sys::MenuLayer,
}

/// The persistent data of a [`SimpleMenuWindow`]: its sections, which the firmware points into, and
/// your `D`.
#[repr(C)]
pub struct SimpleMenuData<S, D> {
	// read at offset 0 through the firmware's context pointer: must stay the first field
	header:    SimpleMenuHeader,
	on_select: for<'n> fn(&'n mut D, MenuIndex, &mut UiQueue<'n>),
	sections:  S,
	data:      D,
}

impl<S, D> SimpleMenuData<S, D> {
	/// Your data.
	pub fn data(&mut self) -> &mut D {
		&mut self.data
	}

	/// The menu's sections.
	pub fn sections(&self) -> &S {
		&self.sections
	}
}

/// The callback of every [`MenuItem`]. The firmware passes the row but not its section, so the
/// index comes from the selection, which Select and touch both commit before calling this.
unsafe extern "C" fn simple_menu_select(_row: c_int, context: *mut c_void) {
	// SAFETY: context is a SimpleMenuWindow's SimpleMenuData, which starts with its header
	let (dispatch, menu) = unsafe {
		let header = context.cast::<SimpleMenuHeader>();
		((*header).dispatch, (*header).menu)
	};
	// SAFETY: the menu lives as long as the window whose callback this is
	let index = unsafe { sys::menu_layer_get_selected_index(menu) }.into();
	// SAFETY: dispatch was written for this window's data type
	unsafe { dispatch(context, index) };
}

/// # Safety
/// `context` must be a live `SimpleMenuData<S, D>`, with no reference into it held.
unsafe fn dispatch<S, D>(context: *mut c_void, index: MenuIndex) {
	let mut ui = UiGuard::enter_internal();
	let data = context.cast::<SimpleMenuData<S, D>>();
	// SAFETY: the guard excludes every other borrow into the window. Only D is borrowed: the
	// firmware reads the sections, which a queued reload does once this returns
	let (on_select, menu, data) =
		unsafe { ((*data).on_select, (*data).header.menu, &mut (*data).data) };
	// SAFETY: written by new(), from the firmware's non-null menu
	let menu = unsafe { NonNull::new_unchecked(menu) };
	on_select(data, index, &mut UiQueue::for_menu(&mut ui, menu));
}

/// A window showing a fixed list of rows. The firmware counts and draws them from `S`, so a
/// `static` table of [`MenuSection`]s costs the app no drawing code, and Select hands the chosen
/// [`MenuIndex`] to one function: a `fn`, or a closure that captures nothing, keeping its state in
/// `D` instead. Rows only known at runtime go in a [`MenuList`]. For rows that change, or are
/// custom drawn, use a [`MenuWindow`](super::MenuWindow).
///
/// The firmware keeps pointers into the sections for the window's life, so they're fixed when it's
/// made: the window owns `S`. As with a `MenuWindow`, the menu owns the buttons, so it has no click
/// handlers.
///
/// ```no_run
/// # use gneiss::ui::{MenuItem, MenuSection, SimpleMenuWindow};
/// static FRUIT: [MenuSection; 1] = [MenuSection::new(&[
///     MenuItem::new(c"Plum"),
///     MenuItem::new(c"Strawberry").subtitle(c"not a berry"),
/// ])];
///
/// # fn main() {
/// let window = SimpleMenuWindow::new(&FRUIT, (), |_, index, queue| {
///     if index.row == 0 {
///         queue.pop(true);
///     }
/// });
/// # }
/// ```
pub struct SimpleMenuWindow<S, D = ()>(Window<NoLayout<SimpleMenuData<S, D>>>)
where
	S: Sections + 'static,
	D: 'static;

impl<S: Sections, D> SimpleMenuWindow<S, D> {
	/// Create a window showing `sections`, calling `on_select` with `data` when a row is
	/// selected. `None` if the firmware is out of memory.
	pub fn new(
		sections: S,
		data: D,
		on_select: for<'n> fn(&'n mut D, MenuIndex, &mut UiQueue<'n>),
	) -> Option<Self> {
		let window = Window::new(Some(SimpleMenuData {
			header: SimpleMenuHeader {
				dispatch: dispatch::<S, D>,
				menu:     ptr::null_mut(),
			},
			on_select,
			sections,
			data,
		}))?;
		let ptr = window.ptr;
		// SAFETY: the window was just made, so nothing borrows into it. Its WindowData is boxed, so
		// the sections the firmware points into never move, and they're freed only after the menu
		unsafe {
			let data: *mut SimpleMenuData<S, D> =
				(*Window::<NoLayout<SimpleMenuData<S, D>>>::window_data(ptr))
					.data
					.as_mut()
					.unwrap_unchecked();
			let sections = (*data).sections.sections();
			let root = sys::window_get_root_layer(ptr);
			let simple = NonNull::new(sys::simple_menu_layer_create(
				sys::layer_get_bounds(root),
				ptr,
				sections.as_ptr().cast(),
				sections.len() as i32,
				data.cast(),
			))?;
			let menu = sys::simple_menu_layer_get_menu_layer(simple.as_ptr());
			(*data).header.menu = menu;
			// Destroyed with menu_layer_destroy, which frees the whole SimpleMenuLayer: its
			// MenuLayer is its first field, and simple_menu_layer_destroy does the same
			(*header(ptr)).menu = Some(NonNull::new_unchecked(menu));
			sys::layer_add_child(root, sys::menu_layer_get_layer(menu));
		}
		Some(Self(window))
	}

	/// The window's [`WindowCtx`], from outside its callbacks.
	pub fn as_ctx<'g>(&'g mut self, ui: &'g mut UiGuard) -> WindowCtx<'g, SimpleMenuData<S, D>> {
		self.0.as_ctx(ui)
	}

	/// The window's menu, from outside its callbacks.
	pub fn menu<'g>(&'g mut self, _ui: &'g mut UiGuard) -> MenuRef<'g> {
		// SAFETY: new() set the menu, which lives as long as the window; &mut self and the guard
		// exclude every other borrow into it
		unsafe { MenuRef::from_raw((*header(self.0.ptr)).menu.unwrap_unchecked()) }
	}

	/// Leak this window and return a handle to it. Afterwards only the handle and the window's own
	/// callbacks can reach it, so keep the handle, e.g. in a [`StaticWindow`](super::StaticWindow).
	#[must_use]
	pub fn leak(self) -> AnyWindow<'static> {
		self.0.leak()
	}
}

// SAFETY: leaks the inner Window<NoLayout<SimpleMenuData<S, D>>>
unsafe impl<S: Sections, D> LeakWindow<NoLayout<SimpleMenuData<S, D>>> for SimpleMenuWindow<S, D> {
	fn leak(self) -> AnyWindow<'static> {
		self.0.leak()
	}
}

impl<'w, S: Sections, D> From<&'w SimpleMenuWindow<S, D>> for AnyWindow<'w> {
	fn from(window: &'w SimpleMenuWindow<S, D>) -> Self {
		(&window.0).into()
	}
}

impl<'w, S: Sections, D> TypedWindow<'w> for &'w SimpleMenuWindow<S, D> {
	type Layout = NoLayout<SimpleMenuData<S, D>>;
}
