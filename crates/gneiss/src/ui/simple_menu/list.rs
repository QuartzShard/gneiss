//! [`MenuList`], the rows of a [`SimpleMenuWindow`](super::SimpleMenuWindow) built at runtime.
use core::{ffi::CStr, ptr};

use super::{MenuSection, Sections, sealed::Sealed, simple_menu_select};
use crate::{
	alloc::{Arena, ArenaFill, Handle},
	sys,
};

/// The rows of a [`SimpleMenuWindow`](super::SimpleMenuWindow), built at runtime: its sections,
/// rows and a copy of every string, packed into one [`Arena`] of exactly the size needed.
///
/// ```no_run
/// # use core::fmt::Write;
/// # use gneiss::ui::{MenuList, SimpleMenuWindow, StackText};
/// # fn main() {
/// let laps = [83_450u32, 81_920, 85_010];
/// let list = MenuList::new(|list| {
///     list.section(Some(c"Laps"));
///     for (lap, ms) in laps.iter().enumerate() {
///         let mut title = StackText::<12>::new();
///         let _ = write!(title, "Lap {}", lap + 1);
///         let mut time = StackText::<12>::new();
///         let _ = write!(time, "{}.{:03}s", ms / 1000, ms % 1000);
///         list.item(&title, Some(&time));
///     }
/// })
/// .expect("out of memory");
/// let window = SimpleMenuWindow::new(list, (), |_, _, queue| {
///     queue.pop(true);
/// });
/// # }
/// ```
pub struct MenuList {
	arena:    Arena,
	sections: Handle<[sys::SimpleMenuSection]>,
}

/// What [`MenuList::new`]'s function fills the list through.
pub struct MenuListFill<'f> {
	fill:         &'f mut ArenaFill,
	sections:     Handle<[sys::SimpleMenuSection]>,
	items:        Handle<[sys::SimpleMenuItem]>,
	num_sections: usize,
	num_items:    usize,
}

impl MenuList {
	/// Build a list with `fill`, which adds its sections and rows.
	///
	/// `fill` runs twice, first to count and then to write, so it must add the same things in the
	/// same order both times, and any work it does is done twice, as for [`Arena::new_exact`].
	///
	/// `None` if the firmware is out of memory, or the two runs differ.
	pub fn new(fill: impl Fn(&mut MenuListFill<'_>)) -> Option<Self> {
		let mut measure = ArenaFill::measuring();
		let mut count = MenuListFill::new(&mut measure, 0, 0);
		fill(&mut count);
		let (num_sections, num_items) = (count.num_sections, count.num_items);
		let capacity = size_of::<sys::SimpleMenuSection>()
			.checked_mul(num_sections)?
			.checked_add(size_of::<sys::SimpleMenuItem>().checked_mul(num_items)?)?
			.checked_add(measure.used())?;
		let mut block = Arena::new_with_capacity(capacity)?;
		let mut write = MenuListFill::new(&mut block, num_sections, num_items);
		fill(&mut write);
		if (write.num_sections, write.num_items) != (num_sections, num_items) {
			return None;
		}
		let sections = write.sections;
		Some(Self {
			arena: block.finish()?,
			sections,
		})
	}
}

impl Sealed for MenuList {}

impl Sections for MenuList {
	fn sections(&self) -> &[MenuSection<'_>] {
		// SAFETY: MenuSection is a transparent SimpleMenuSection. new() wrote every section, and
		// they and everything they point to live in the arena, as long as self
		unsafe {
			core::slice::from_raw_parts(self.arena.ptr(self.sections).cast(), self.sections.len())
		}
	}
}

impl<'f> MenuListFill<'f> {
	fn new(fill: &'f mut ArenaFill, num_sections: usize, num_items: usize) -> Self {
		Self {
			sections: fill.reserve_slice(num_sections),
			items: fill.reserve_slice(num_items),
			fill,
			num_sections: 0,
			num_items: 0,
		}
	}

	/// Start a section, with a header showing `title` if given. Rows added before the first
	/// section go in one without a header.
	pub fn section(&mut self, title: Option<&CStr>) {
		let title = title.map_or(ptr::null(), |title| self.push(title));
		let items = self.start(self.items);
		if let Some(section) = self.slot(self.sections, self.num_sections) {
			// SAFETY: a reserved section in the block
			unsafe {
				section.write(sys::SimpleMenuSection {
					title,
					items: items.wrapping_add(self.num_items),
					num_items: 0,
				});
			}
		}
		self.num_sections += 1;
	}

	/// Add a row showing `title`, and `subtitle` under it if given.
	pub fn item(&mut self, title: &CStr, subtitle: Option<&CStr>) {
		if self.num_sections == 0 {
			self.section(None);
		}
		let title = self.push(title);
		let subtitle = subtitle.map_or(ptr::null(), |subtitle| self.push(subtitle));
		if let Some(item) = self.slot(self.items, self.num_items) {
			// SAFETY: a reserved item in the block
			unsafe {
				item.write(sys::SimpleMenuItem {
					title,
					subtitle,
					icon: ptr::null_mut(),
					callback: Some(simple_menu_select),
				});
			}
		}
		if let Some(section) = self.slot(self.sections, self.num_sections - 1) {
			// SAFETY: the current section, written by section()
			unsafe { (*section).num_items += 1 };
		}
		self.num_items += 1;
	}

	fn push(&mut self, s: &CStr) -> *const core::ffi::c_char {
		let handle = self.fill.push_c_str(s);
		self.fill.ptr(handle).cast()
	}

	// Element `index` of `region`, while writing and if it was counted
	fn slot<T>(&self, region: Handle<[T]>, index: usize) -> Option<*mut T> {
		(!self.fill.is_measuring() && index < region.len())
			.then(|| self.start(region).wrapping_add(index))
	}

	fn start<T>(&self, region: Handle<[T]>) -> *mut T {
		self.fill.ptr(region).cast_mut().cast()
	}
}
