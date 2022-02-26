use std::any::TypeId;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU32;

mod attributes;
mod builder;
mod flags;
mod parents;

pub use attributes::Attributes;
pub use builder::Builder;
pub use flags::Flags;
pub use parents::{HasParents, Parents};

#[repr(C, align(8))]
#[derive(Debug)]
pub struct Base<T: 'static> { // TODO: rename me to Allocated
	parents: UnsafeCell<Parents>,
	attributes: Option<Box<Attributes>>,
	pub(super) typeid: TypeId,
	pub(super) flags: Flags,
	borrows: AtomicU32,
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
		&self.flags
	}

	pub fn typeid(&self) -> TypeId {
		self.typeid
	}

	pub unsafe fn upcast(data: *const T) -> *const Self {
		container_of::container_of!(data, Self, data)
	}
}

impl<T> Drop for Base<T> {
	fn drop(&mut self) {
		// TODO: drop data.
	}
}
