use super::{Base, Parents};
use crate::value::gc::{Gc, GcRef, GcMut};
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of_mut, NonNull};

#[must_use]
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T: 'static> Builder<T> {
	pub unsafe fn new(parents: Parents) -> Self {
		let layout = std::alloc::Layout::new::<Base<T>>();

		// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
		// and `attribtues` is valid (as it's zero, which is also `None`).
		let ptr = NonNull::new_unchecked(crate::alloc_zeroed(layout).cast::<Base<T>>());

		// Everything else is default initialized to zero.
		addr_of_mut!((*ptr.as_ptr()).typeid).write(TypeId::of::<T>());
		addr_of_mut!((*ptr.as_ptr()).parents).write(parents.into());

		Self(ptr)
	}

	#[inline]
	pub fn base(&self) -> &Base<T> {
		unsafe { self.0.as_ref() }
	}

	#[inline]
	pub fn base_mut(&mut self) -> &mut Base<T> {
		unsafe { self.0.as_mut() }
	}

	pub fn flags(&self) -> &super::Flags {
		&self.base().flags()
	}

	pub fn data(&self) -> &MaybeUninit<T> {
		unsafe { &*self.base().data.get() }
	}

	pub fn data_mut(&mut self) -> &mut MaybeUninit<T> {
		self.base_mut().data.get_mut()
	}

	pub unsafe fn finish(self) -> Gc<T> {
		Gc::_new(self.0)
	}

	pub unsafe fn gcmut(&mut self) -> &mut GcMut<T> {
		std::mem::transmute(self)
	}

	pub unsafe fn gcref(&self) -> &GcRef<T> {
		std::mem::transmute(self)
	}
}
