use super::{Base, Parents};
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of_mut, NonNull};

#[must_use]
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T> Builder<T> {
	// safety: among other things, `ptr` must have been zero initialized (or you have to init it all yourself)
	pub unsafe fn new(ptr: NonNull<Base<T>>) -> Self {
		addr_of_mut!((*ptr.as_ptr()).header.typeid).write(TypeId::of::<T>());

		Self(ptr)
	}

	pub fn allocate() -> Self {
		let layout = std::alloc::Layout::new::<Base<T>>();

		unsafe {
			// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
			// and `attribtues` is valid (as it's zero, which is also `None`).
			Self::new(NonNull::new_unchecked(
				crate::alloc_zeroed(layout).cast::<Base<T>>(),
			))
		}
	}

	pub unsafe fn _write_parents(&mut self, parents: Parents) {
		addr_of_mut!((*self.0.as_ptr()).header.attributes.parents).write(parents);
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

	pub unsafe fn finish(self) -> NonNull<Base<T>> {
		self.0
	}
}
