use std::any::TypeId;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU32;
use std::ptr::NonNull;

mod attributes;
mod builder;
mod flags;

use attributes::Attributes;
pub use attributes::Parents;
pub use builder::Builder;
pub use flags::Flags;

pub trait HasParents {
	unsafe fn init();
	fn parents() -> Parents;
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Header {
	pub(super) attributes: Attributes,
	pub(super) typeid: TypeId,
	pub(super) flags: Flags,
	pub(super) borrows: AtomicU32,
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Base<T: 'static> {
	pub(super) header: Header,
	pub(super) data: UnsafeCell<MaybeUninit<T>>,
}

impl<T: HasParents + crate::value::gc::Allocated> Base<T> {
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

	pub unsafe fn builder_inplace(base: NonNull<Self>) -> Builder<T> {
		let mut b = Builder::new(base);
		b._write_parents(T::parents());
		b
	}

	pub unsafe fn static_builder(base: &'static mut MaybeUninit<Self>) -> Builder<T> {
		let builder = Self::builder_inplace(NonNull::new_unchecked(base.as_mut_ptr()));
		builder.flags().insert(Flags::NOFREE);
		builder
	}
}

impl<T: crate::value::gc::Allocated> Base<T> {
	pub unsafe fn allocate_with_parents(parents: Parents) -> Builder<T> {
		let mut b = Builder::allocate();
		b._write_parents(parents);
		b
	}
}

impl<T> Base<T> {
	pub const fn flags(&self) -> &Flags {
		&self.header.flags
	}

	pub const fn typeid(&self) -> TypeId {
		self.header.typeid
	}

	pub const fn header(&self) -> &Header {
		&self.header
	}

	pub const fn data(&self) -> &T {
		unsafe { (*(self.data.get() as *const MaybeUninit<T>)).assume_init_ref() }
	}

	pub fn data_mut(&mut self) -> &mut T {
		unsafe { (*self.data.get()).assume_init_mut() }
	}
}

impl<T> Drop for Base<T> {
	fn drop(&mut self) {
		// TODO: drop data.
	}
}
