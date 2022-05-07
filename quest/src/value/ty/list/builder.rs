use super::{Inner, List, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::Builder as BaseBuilder;
use crate::value::gc::Gc;
use crate::value::HasDefaultParent;

#[must_use]
pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	pub unsafe fn new(mut builder: BaseBuilder<Inner>) -> Self {
		builder.set_parents(Gc::<List>::parent());

		Self(builder)
	}

	pub fn allocate() -> Self {
		unsafe { Self::new(BaseBuilder::<Inner>::allocate()) }
	}

	pub fn insert_flags(&mut self, flag: u32) {
		self.0.insert_user_flags(flag);
	}

	pub unsafe fn inner_mut(&mut self) -> &mut Inner {
		&mut *self.0.data_mut()
	}

	pub unsafe fn list_mut(&mut self) -> &mut List {
		&mut *self.0.base_mut().cast::<List>()
	}

	// pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
	// 	self.0.gcmut().as_mut_ptr()
	// }

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate_buffer(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flags(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.inner_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity))
			.as_ptr()
			.cast();
	}

	#[must_use]
	pub unsafe fn finish(self) -> Gc<List> {
		Gc::from_inner(self.0.finish())
	}
}
