use std::any::TypeId;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU32;

mod attributes;
mod builder;
mod flags;

use attributes::Attributes;
pub use attributes::Parents;
pub use builder::Builder;
pub use flags::Flags;

pub trait HasParents {
	fn parents() -> Parents;
}

pub struct Header {
	pub(super) attributes: Attributes,
	pub(super) typeid: TypeId,
	pub(super) flags: Flags,
	pub(super) borrows: AtomicU32,
}

#[repr(C, align(8))]
// #[derive(Debug)]
pub struct Base<T: 'static> {
	// TODO: rename me to Allocated
	pub(super) header: Header,
	pub(super) data: UnsafeCell<MaybeUninit<T>>,
}

impl<T: HasParents + 'static> Base<T> {
	pub fn new(data: T) -> crate::value::Gc<T> {
		unsafe {
			let mut builder = Self::allocate();
			builder.data_mut().write(data);
			builder.finish()
		}
	}

	pub unsafe fn allocate() -> Builder<T> {
		Self::allocate_with_parents(T::parents())
	}
}

impl<T: 'static> Base<T> {
	pub unsafe fn allocate_with_parents(parents: Parents) -> Builder<T> {
		Builder::new(parents)
	}

	pub fn flags(&self) -> &Flags {
		&self.header.flags
	}

	pub fn typeid(&self) -> TypeId {
		self.header.typeid
	}

	pub fn header(&self) -> &Header {
		&self.header
	}
}

impl<T> Drop for Base<T> {
	fn drop(&mut self) {
		// TODO: drop data.
	}
}
