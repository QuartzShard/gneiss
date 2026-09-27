use crate::{
	resource::{ImageKey, PdcKey},
	sys,
};

/// An image loaded at runtime
pub struct GBitMap {
	inner: *mut sys::GBitmap,
}
impl GBitMap {
	pub fn load(key: ImageKey) -> Option<Self> {
		let inner = unsafe { sys::gbitmap_create_with_resource(key.0.get()) };
		if inner.is_null() {
  			None
  		} else {
  			Some(GBitMap { inner })
  		}
	}
}
impl Drop for GBitMap {
	fn drop(&mut self) {
		unsafe { sys::gbitmap_destroy(self.inner) }
	}
}

pub struct DrawCommand {
	inner: *mut sys::GDrawCommandImage,
}
impl DrawCommand {
	pub fn load(key: PdcKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_image_create_with_resource(key.0.get()) };
		if inner.is_null() {
  			None
  		} else {
  			Some(DrawCommand { inner })
  		}
	}
}
impl Drop for DrawCommand {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_image_destroy(self.inner) }
	}
}

pub struct DrawCommandSequence {
	inner: *mut sys::GDrawCommandSequence,
}
impl DrawCommandSequence {
	pub fn load(key: PdcKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_sequence_create_with_resource(key.0.get()) };
		if inner.is_null() {
  			None
  		} else {
  			Some(DrawCommandSequence { inner })
  		}
	}
}
impl Drop for DrawCommandSequence {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_sequence_destroy(self.inner) }
	}
}
