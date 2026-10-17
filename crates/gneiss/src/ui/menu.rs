//! [`MenuWindow`], a window showing a scrolling menu, and the [`Menu`] that fills it.
use core::{ffi::CStr, marker::PhantomData, ptr::NonNull};

use embedded_graphics_core::primitives::Rectangle;

use crate::{
	sys,
	ui::{
		AnyWindow, AsLayer, Layout, NoLayout, TypedWindow, UiGuard, UiQueue, Window, WindowCtx,
		graphics::{Colour, GContext, rect},
		window::{LeakWindow, header},
	},
};

/// A row's place in a menu: its section, and its row within that section. Orders by section, then
/// row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MenuIndex {
	pub section: u16,
	pub row:     u16,
}

impl From<sys::MenuIndex> for MenuIndex {
	fn from(index: sys::MenuIndex) -> Self {
		Self {
			section: index.section,
			row:     index.row,
		}
	}
}

impl From<MenuIndex> for sys::MenuIndex {
	fn from(index: MenuIndex) -> Self {
		Self {
			section: index.section,
			row:     index.row,
		}
	}
}

/// The cell, header, separator or background being drawn.
/// It's a "`Layer`", but not really, and only getting bounds/checking `is_highlighted` is valid, and
/// it's bound to the lifetime of the render call
pub struct MenuCanvas<'r> {
	ptr:     *const sys::Layer,
	_render: PhantomData<&'r sys::Layer>,
}

impl MenuCanvas<'_> {
	/// The cell's bounds, in its own coordinates.
	pub fn bounds(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_bounds(self.ptr) })
	}

	/// Whether the cell is drawn as the selected one.
	pub fn is_highlighted(&self) -> bool {
		unsafe { sys::menu_cell_layer_is_highlighted(self.ptr) }
	}

	/// Draw a standard row: `title`, and `subtitle` in smaller text under it, in the menu's colours.
	pub fn draw_basic(&self, ctx: &mut GContext<'_>, title: &CStr, subtitle: Option<&CStr>) {
		// SAFETY: this is the cell being drawn with ctx; the strings are only read during the call
		unsafe {
			sys::menu_cell_basic_draw(
				ctx.as_ptr(),
				self.ptr,
				title.as_ptr(),
				subtitle.map_or(core::ptr::null(), CStr::as_ptr),
				core::ptr::null_mut(),
			);
		}
	}

	/// Draw a standard section header: `title` in small bold text.
	pub fn draw_basic_header(&self, ctx: &mut GContext<'_>, title: &CStr) {
		// SAFETY: as for draw_basic
		unsafe { sys::menu_cell_basic_header_draw(ctx.as_ptr(), self.ptr, title.as_ptr()) };
	}
}

/// What a [`MenuWindow`] shows and what it does when used. Implement it on a marker type, or on
/// [`Menu::Data`] itself, with `#[gneiss::menu]` on the impl block.
///
/// Only `get_num_rows` and `draw_row` are required. A method left out isn't called: the firmware
/// uses its own default, which the default bodies here describe. `get_header_height` and
/// `draw_header` come as a pair, as do `get_separator_height` and `draw_separator`.
///
/// ```no_run
/// # use gneiss::ui::{Menu, MenuIndex, MenuCanvas, MenuWindow, UiQueue, graphics::GContext};
/// struct Fruit;
///
/// #[gneiss::menu]
/// impl Menu for Fruit {
///     type Data = [&'static str; 3];
///
///     fn get_num_rows(names: &Self::Data, _section: u16) -> u16 {
///         names.len() as u16
///     }
///
///     fn draw_row(
///         names: &Self::Data,
///         cell: &MenuCanvas<'_>,
///         index: MenuIndex,
///         ctx: &mut GContext<'_>,
///     ) {
///         let _ = (names[usize::from(index.row)], cell.bounds(), ctx);
///     }
///
///     fn select_click<'n>(
///         _layout: &'n mut Self::Layout,
///         names: &'n mut Self::Data,
///         index: MenuIndex,
///         _nav: &mut UiQueue<'n>,
///     ) {
///         names[usize::from(index.row)] = "eaten";
///     }
/// }
///
/// # fn main() {
/// let window = MenuWindow::<Fruit>::new(["apple", "pear", "plum"]);
/// # }
/// ```
pub trait Menu: __trampolines::MenuCallbacks {
	/// What the menu shows: the window's persistent data, which every method reads.
	type Data: 'static;

	/// The window's [`Layout`], loaded over the menu, such as a status bar. Select is handed it.
	type Layout: Layout<Data = Self::Data> + 'static = NoLayout<Self::Data>;

	/// How many sections the menu has.
	fn get_num_sections(_data: &Self::Data) -> u16 {
		1
	}

	/// How many rows `section` has.
	fn get_num_rows(data: &Self::Data, section: u16) -> u16;

	/// The height of the row at `index`. Left out, the firmware's basic cell height is used, which
	/// follows the platform and the user's text size; this default body is that height at the
	/// default size.
	fn get_cell_height(_data: &Self::Data, _index: MenuIndex) -> i16 {
		44
	}

	/// The height of `section`'s header. Left out, sections have no header.
	fn get_header_height(_data: &Self::Data, _section: u16) -> i16 {
		0
	}

	/// The height of the separator below the row at `index`. Left out, rows have none.
	fn get_separator_height(_data: &Self::Data, _index: MenuIndex) -> i16 {
		0
	}

	/// Draw the row at `index` into `cell`.
	fn draw_row(data: &Self::Data, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>);

	/// Draw `section`'s header into `cell`.
	fn draw_header(
		_data: &Self::Data,
		_cell: &MenuCanvas<'_>,
		_section: u16,
		_ctx: &mut GContext<'_>,
	) {
	}

	/// Draw the separator below the row at `index` into `cell`.
	fn draw_separator(
		_data: &Self::Data,
		_cell: &MenuCanvas<'_>,
		_index: MenuIndex,
		_ctx: &mut GContext<'_>,
	) {
	}

	/// Draw behind the rows, highlighted or not to match them. Left out, the menu's normal and
	/// highlight colours fill it.
	fn draw_background(
		_data: &Self::Data,
		_cell: &MenuCanvas<'_>,
		_highlight: bool,
		_ctx: &mut GContext<'_>,
	) {
	}

	/// Select was pressed on the row at `index`. Left out, Select is ignored. After changing which
	/// rows there are, [`queue.reload()`](UiQueue::reload) re-reads them once this returns.
	fn select_click<'n>(
		_layout: &'n mut Self::Layout,
		_data: &'n mut Self::Data,
		_index: MenuIndex,
		_nav: &mut UiQueue<'n>,
	) {
	}

	/// Select was held on the row at `index`. Left out, Select isn't subscribed to long clicks, so a
	/// short press doesn't wait to rule one out.
	fn select_long_click<'n>(
		_layout: &'n mut Self::Layout,
		_data: &'n mut Self::Data,
		_index: MenuIndex,
		_nav: &mut UiQueue<'n>,
	) {
	}

	/// The selection moved from `old` to `new`.
	fn selection_changed(_data: &mut Self::Data, _new: MenuIndex, _old: MenuIndex) {}

	/// The selection is about to move from `old` to `new`: the index to move to instead. Returning
	/// `old` keeps it where it is.
	fn selection_will_change(_data: &Self::Data, new: MenuIndex, _old: MenuIndex) -> MenuIndex {
		new
	}
}

/// The `extern "C"` callbacks `#[gneiss::menu]` fills `MenuCallbacks::CALLBACKS` with. The
/// firmware's context is the window's `WindowData`: each reaches the menu's `Data` through it.
///
/// Only the draws and selects take the [`UiGuard`]. The rest fire synchronously inside menu calls
/// made through a [`MenuRef`](super::MenuRef), which may run while the guard is held.
#[doc(hidden)]
pub mod __trampolines {
	use core::{ffi::c_void, ptr::NonNull};

	use super::{Menu, MenuCanvas, MenuIndex};
	use crate::{
		sys,
		ui::{UiGuard, UiQueue, graphics::GContext, window::WindowData},
	};

	/// The firmware callbacks for a [`Menu`], written by `#[gneiss::menu]`.
	#[diagnostic::on_unimplemented(
		message = "`{Self}` implements `Menu` without `#[gneiss::menu]`",
		label = "needs `#[gneiss::menu]` on its `impl Menu` block",
		note = "`#[gneiss::menu]` writes the firmware callbacks for the methods the impl defines"
	)]
	pub trait MenuCallbacks {
		const CALLBACKS: sys::MenuLayerCallbacks;
	}

	/// Every callback unset, for the ones a `Menu` impl leaves out
	pub const NONE: sys::MenuLayerCallbacks = sys::MenuLayerCallbacks {
		get_num_sections:      None,
		get_num_rows:          None,
		get_cell_height:       None,
		get_header_height:     None,
		draw_row:              None,
		draw_header:           None,
		select_click:          None,
		select_long_click:     None,
		selection_changed:     None,
		get_separator_height:  None,
		draw_separator:        None,
		selection_will_change: None,
		draw_background:       None,
	};

	/// # Safety
	/// `context` must be a live `MenuWindow<M>`'s context.
	unsafe fn window<M: Menu>(context: *mut c_void) -> *mut WindowData<M::Layout> {
		context.cast()
	}

	/// # Safety
	/// `context` must be a live `MenuWindow<M>`'s context, with no `&mut` to its data held.
	unsafe fn data<'a, M: Menu>(context: *mut c_void) -> &'a M::Data {
		// SAFETY: MenuWindow::new stores Some, and nothing can set None
		unsafe { (*window::<M>(context)).data.as_ref().unwrap_unchecked() }
	}

	/// # Safety
	/// `context` must be a live `MenuWindow<M>`'s context, with no reference to its data held.
	unsafe fn data_mut<'a, M: Menu>(context: *mut c_void) -> &'a mut M::Data {
		// SAFETY: as for data
		unsafe { (*window::<M>(context)).data.as_mut().unwrap_unchecked() }
	}

	/// # Safety
	/// `menu`, `index` and `context` must be a menu select callback's arguments.
	unsafe fn select<M: Menu>(
		menu: *mut sys::MenuLayer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
		f: for<'n> fn(&'n mut M::Layout, &'n mut M::Data, MenuIndex, &mut UiQueue<'n>),
	) {
		let mut ui = UiGuard::enter_internal();
		let index = unsafe { *index }.into();
		// SAFETY: the guard excludes every other borrow into the window, and these are disjoint
		// fields
		let (layout, data) = unsafe {
			let w = window::<M>(context);
			(&mut (*w).layout, data_mut::<M>(context))
		};
		// Select only fires while the window is on screen, so loaded
		let Some(layout) = layout else { return };
		// SAFETY: the firmware passes the menu whose callback this is
		let menu = unsafe { NonNull::new_unchecked(menu) };
		f(layout, data, index, &mut UiQueue::for_menu(&mut ui, menu));
	}

	/// # Safety
	/// `ctx` and `cell` must be a menu draw callback's arguments, and `context` as for `data`.
	unsafe fn draw<M: Menu>(
		ctx: *mut sys::GContext,
		cell: *const sys::Layer,
		context: *mut c_void,
		f: impl FnOnce(&M::Data, &MenuCanvas<'_>, &mut GContext<'_>),
	) {
		let _ui = UiGuard::enter_internal();
		let mut ctx = unsafe { GContext::new(ctx, cell.cast_mut()) };
		let cell = MenuCanvas {
			ptr:     cell,
			_render: core::marker::PhantomData,
		};
		f(unsafe { data::<M>(context) }, &cell, &mut ctx);
	}

	pub unsafe extern "C" fn get_num_sections<M: Menu>(
		_menu: *mut sys::MenuLayer,
		context: *mut c_void,
	) -> u16 {
		M::get_num_sections(unsafe { data::<M>(context) })
	}

	pub unsafe extern "C" fn get_num_rows<M: Menu>(
		_menu: *mut sys::MenuLayer,
		section: u16,
		context: *mut c_void,
	) -> u16 {
		M::get_num_rows(unsafe { data::<M>(context) }, section)
	}

	pub unsafe extern "C" fn get_cell_height<M: Menu>(
		_menu: *mut sys::MenuLayer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) -> i16 {
		M::get_cell_height(unsafe { data::<M>(context) }, unsafe { *index }.into())
	}

	pub unsafe extern "C" fn get_header_height<M: Menu>(
		_menu: *mut sys::MenuLayer,
		section: u16,
		context: *mut c_void,
	) -> i16 {
		M::get_header_height(unsafe { data::<M>(context) }, section)
	}

	pub unsafe extern "C" fn get_separator_height<M: Menu>(
		_menu: *mut sys::MenuLayer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) -> i16 {
		M::get_separator_height(unsafe { data::<M>(context) }, unsafe { *index }.into())
	}

	pub unsafe extern "C" fn draw_row<M: Menu>(
		ctx: *mut sys::GContext,
		cell: *const sys::Layer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) {
		let index = unsafe { *index }.into();
		unsafe {
			draw::<M>(ctx, cell, context, |data, cell, ctx| {
				M::draw_row(data, cell, index, ctx);
			});
		}
	}

	pub unsafe extern "C" fn draw_header<M: Menu>(
		ctx: *mut sys::GContext,
		cell: *const sys::Layer,
		section: u16,
		context: *mut c_void,
	) {
		unsafe {
			draw::<M>(ctx, cell, context, |data, cell, ctx| {
				M::draw_header(data, cell, section, ctx);
			});
		}
	}

	pub unsafe extern "C" fn draw_separator<M: Menu>(
		ctx: *mut sys::GContext,
		cell: *const sys::Layer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) {
		let index = unsafe { *index }.into();
		unsafe {
			draw::<M>(ctx, cell, context, |data, cell, ctx| {
				M::draw_separator(data, cell, index, ctx);
			});
		}
	}

	pub unsafe extern "C" fn draw_background<M: Menu>(
		ctx: *mut sys::GContext,
		cell: *const sys::Layer,
		highlight: bool,
		context: *mut c_void,
	) {
		unsafe {
			draw::<M>(ctx, cell, context, |data, cell, ctx| {
				M::draw_background(data, cell, highlight, ctx);
			});
		}
	}

	pub unsafe extern "C" fn select_click<M: Menu>(
		menu: *mut sys::MenuLayer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) {
		unsafe { select::<M>(menu, index, context, M::select_click) };
	}

	pub unsafe extern "C" fn select_long_click<M: Menu>(
		menu: *mut sys::MenuLayer,
		index: *mut sys::MenuIndex,
		context: *mut c_void,
	) {
		unsafe { select::<M>(menu, index, context, M::select_long_click) };
	}

	pub unsafe extern "C" fn selection_changed<M: Menu>(
		_menu: *mut sys::MenuLayer,
		new: sys::MenuIndex,
		old: sys::MenuIndex,
		context: *mut c_void,
	) {
		M::selection_changed(unsafe { data_mut::<M>(context) }, new.into(), old.into());
	}

	pub unsafe extern "C" fn selection_will_change<M: Menu>(
		_menu: *mut sys::MenuLayer,
		new: *mut sys::MenuIndex,
		old: sys::MenuIndex,
		context: *mut c_void,
	) {
		unsafe {
			*new = M::selection_will_change(data::<M>(context), (*new).into(), old.into()).into();
		}
	}
}

/// Where a newly selected row scrolls to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MenuAlign {
	None   = sys::MenuRowAlign_MenuRowAlignNone,
	Center = sys::MenuRowAlign_MenuRowAlignCenter,
	Top    = sys::MenuRowAlign_MenuRowAlignTop,
	Bottom = sys::MenuRowAlign_MenuRowAlignBottom,
}

/// A window showing a menu, with the rows its [`Menu`] gives. The menu fills the window and lives
/// as long as it does. [`Menu::Layout`] loads over it like any window's layout: a layer it adds,
/// such as a status bar, draws on top, and it can make room by moving the menu through
/// [`WindowCtx::menu`].
///
/// The menu owns the buttons: Up and Down move the selection, Select calls
/// [`Menu::select_click`], and Back pops the window. So unlike a [`Window`], a `MenuWindow` has no
/// click handlers.
pub struct MenuWindow<M: Menu>(Window<M::Layout>);

impl<M: Menu> MenuWindow<M> {
	/// Create a window showing a menu of `data`. `None` if the firmware is out of memory.
	pub fn new(data: M::Data) -> Option<Self> {
		let window = Window::new(Some(data))?;
		let ptr = window.ptr;
		// SAFETY: the window was just created, so nothing else borrows into it. set_callbacks reads
		// the data straight away, through the context
		unsafe {
			let root = sys::window_get_root_layer(ptr);
			let menu = NonNull::new(sys::menu_layer_create(sys::layer_get_bounds(root)))?;
			(*header(ptr)).menu = Some(menu);
			sys::menu_layer_set_callbacks(
				menu.as_ptr(),
				sys::window_get_user_data(ptr),
				M::CALLBACKS,
			);
			sys::layer_add_child(root, menu.as_ptr().cast());
			sys::menu_layer_set_click_config_onto_window(menu.as_ptr(), ptr);
		}
		Some(Self(window))
	}

	/// The window's [`WindowCtx`], from outside its callbacks.
	pub fn as_ctx<'g>(&'g mut self, ui: &'g mut UiGuard) -> WindowCtx<'g, M::Data> {
		self.0.as_ctx(ui)
	}

	/// The window's layout. `None` while it is unloaded.
	pub fn layout<'g>(&'g mut self, ui: &'g mut UiGuard) -> Option<&'g mut M::Layout> {
		self.0.layout(ui)
	}

	/// The window's menu, from outside its callbacks.
	pub fn menu<'g>(&'g mut self, _ui: &'g mut UiGuard) -> MenuRef<'g> {
		// SAFETY: new() set the menu, which lives as long as the window
		MenuRef {
			ptr:     unsafe { (*header(self.0.ptr)).menu.unwrap_unchecked() },
			_window: PhantomData,
		}
	}

	/// Leak this window and return a handle to it. Afterwards only the handle and the window's own
	/// callbacks can reach it, so keep the handle, e.g. in a [`StaticWindow`](super::StaticWindow).
	#[must_use]
	pub fn leak(self) -> AnyWindow<'static> {
		self.0.leak()
	}
}

// SAFETY: leaks the inner Window<M::Layout>
unsafe impl<M: Menu> LeakWindow<M::Layout> for MenuWindow<M> {
	fn leak(self) -> AnyWindow<'static> {
		self.0.leak()
	}
}

impl<'w, M: Menu> From<&'w MenuWindow<M>> for AnyWindow<'w> {
	fn from(window: &'w MenuWindow<M>) -> Self {
		(&window.0).into()
	}
}

impl<'w, M: Menu> TypedWindow<'w> for &'w MenuWindow<M> {
	type Layout = M::Layout;
}

impl<D> WindowCtx<'_, D> {
	/// The window's menu, if it's a [`MenuWindow`].
	pub fn menu(&mut self) -> Option<MenuRef<'_>> {
		// SAFETY: the ctx is for a live gneiss window, and only Window::new and MenuWindow::new
		// write its menu
		let ptr = unsafe { (*header(self.ptr())).menu }?;
		Some(MenuRef {
			ptr,
			_window: PhantomData,
		})
	}
}

/// A [`MenuWindow`]'s menu, borrowed from its window. Its methods can fire the [`Menu`]'s callbacks
/// before they return, which read the window's data: the borrow keeps that data from being borrowed
/// alongside.
pub struct MenuRef<'m> {
	ptr:     NonNull<sys::MenuLayer>,
	_window: PhantomData<&'m mut ()>,
}

// SAFETY: a MenuLayer starts with its ScrollLayer, which starts with its Layer
unsafe impl AsLayer for MenuRef<'_> {
	fn as_ptr(&self) -> *mut sys::Layer {
		self.ptr.as_ptr().cast()
	}
}

impl MenuRef<'_> {
	/// # Safety
	/// `ptr` must be a live menu, and the borrow the result is given must keep its window alive and
	/// exclude other borrows into its data.
	pub(super) unsafe fn from_raw(ptr: NonNull<sys::MenuLayer>) -> Self {
		Self {
			ptr,
			_window: PhantomData,
		}
	}

	/// Re-read the menu's rows, after the data they come from has changed.
	pub fn reload(&mut self) {
		unsafe { sys::menu_layer_reload_data(self.ptr.as_ptr()) };
	}

	/// The selected row.
	pub fn selected(&self) -> MenuIndex {
		unsafe { sys::menu_layer_get_selected_index(self.ptr.as_ptr()) }.into()
	}

	/// Select the row at `index`, clamped to the menu's rows. Doesn't call
	/// [`Menu::selection_will_change`].
	pub fn set_selected(&mut self, index: MenuIndex, align: MenuAlign, animated: bool) {
		unsafe {
			sys::menu_layer_set_selected_index(
				self.ptr.as_ptr(),
				index.into(),
				align as sys::MenuRowAlign,
				animated,
			);
		}
	}

	/// Move the selection one row up or down, as the buttons do.
	pub fn select_next(&mut self, up: bool, align: MenuAlign, animated: bool) {
		unsafe {
			sys::menu_layer_set_selected_next(
				self.ptr.as_ptr(),
				up,
				align as sys::MenuRowAlign,
				animated,
			);
		}
	}

	/// The colours of rows that aren't selected.
	pub fn set_colours(&mut self, background: Colour, foreground: Colour) {
		unsafe {
			sys::menu_layer_set_normal_colors(
				self.ptr.as_ptr(),
				background.into(),
				foreground.into(),
			);
		}
	}

	/// The colours of the selected row.
	pub fn set_highlight_colours(&mut self, background: Colour, foreground: Colour) {
		unsafe {
			sys::menu_layer_set_highlight_colors(
				self.ptr.as_ptr(),
				background.into(),
				foreground.into(),
			);
		}
	}
}
