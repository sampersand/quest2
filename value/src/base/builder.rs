use super::{Base, Parents};
use crate::Gc;
use std::alloc::{self, Layout};
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of_mut, NonNull};

#[must_use]
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T: 'static> Builder<T> {
	pub unsafe fn new(parents: Parents) -> Self {
		let layout = Layout::new::<Base<T>>();

		// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
		// and `attribtues` is valid (as it's zero, which is also `None`).
		let ptr = alloc::alloc_zeroed(layout).cast::<Base<T>>();

		if let Some(nonnull) = NonNull::new(ptr) {
			// Everything else is default initialized to zero.
			addr_of_mut!((*nonnull.as_ptr()).header.typeid).write(TypeId::of::<T>());
			addr_of_mut!((*nonnull.as_ptr()).header.parents).write(parents.into());

			Self(nonnull)
		} else {
			alloc::handle_alloc_error(layout);
		}
	}

	#[inline]
	pub fn base(&self) -> &Base<T> {
		unsafe {
			self.0.as_ref()
		}
	}

	#[inline]
	pub fn base_mut(&mut self) -> &mut Base<T> {
		unsafe {
			self.0.as_mut()
		}
	}

	pub fn flags(&self) -> &super::Flags {
		&self.base().flags()
	}

	pub fn data(&self) -> &MaybeUninit<T> {
		unsafe {
			&*self.base().data.get()
		}
	}

	pub fn data_mut(&mut self) -> &mut MaybeUninit<T> {
		self.base_mut().data.get_mut()
	}

	pub unsafe fn finish(mut self) -> Gc<T> {
		Gc::new(NonNull::new_unchecked(self.data_mut().as_mut_ptr()))
	}
}
