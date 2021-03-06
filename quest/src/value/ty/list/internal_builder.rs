use super::{Inner, List, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::Builder;
use crate::value::gc::Gc;
use crate::value::{HasDefaultParent, HasFlags, HasParents};

#[must_use]
pub(super) struct InternalBuilder(Builder<List>);

impl InternalBuilder {
	pub unsafe fn new(mut builder: Builder<List>) -> Self {
		builder.set_parents(List::parent());

		Self(builder)
	}

	pub fn allocate() -> Self {
		unsafe { Self::new(Builder::new()) }
	}

	pub fn insert_flags(&mut self, flag: u32) {
		self.0.flags().insert_user(flag);
	}

	pub unsafe fn inner_mut(&mut self) -> &mut Inner {
		&mut *self.0.data_mut()
	}

	pub fn list(&self) -> &List {
		unsafe { &*self.0.base().cast::<List>() }
	}

	pub fn list_mut(&mut self) -> &mut List {
		unsafe { &mut *self.0.base_mut().cast::<List>() }
	}

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate_buffer(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flags(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.inner_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity)).as_ptr();
	}

	#[must_use]
	pub unsafe fn finish(self) -> Gc<List> {
		self.0.finish()
	}
}
