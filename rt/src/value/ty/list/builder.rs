use super::{List, ListInner, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::{Builder as BaseBuilder};
use crate::value::gc::{Gc};

pub struct Builder(BaseBuilder<ListInner>);

impl Builder {
	pub unsafe fn new(builder: BaseBuilder<ListInner>) -> Self {
		Self(builder)
	}

	pub fn allocate() -> Self {
		unsafe { Self::new(BaseBuilder::<ListInner>::allocate()) }
	}

	pub fn insert_flag(&mut self, flag: u32) {
		self.0.flags().insert(flag);
	}

	pub unsafe fn inner_mut(&mut self) -> &mut ListInner {
		self.0.data_mut().assume_init_mut()
	}

	pub unsafe fn list_mut(&mut self) -> &mut List {
		std::mem::transmute(self.0.base_mut())
	}

	// pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
	// 	self.0.gcmut().as_mut_ptr()
	// }

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate_buffer(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flag(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.inner_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity)).cast();
	}

	pub unsafe fn finish(self) -> Gc<List> {
		std::mem::transmute(self.0.finish())
	}
}
