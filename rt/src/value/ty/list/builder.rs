use super::{List, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::{Base, Builder as BaseBuilder};
use crate::value::gc::{Gc, GcMut};
use crate::value::AnyValue;

pub struct Builder(BaseBuilder<List>);

impl Builder {
	pub fn new() -> Self {
		Self(unsafe { Base::<List>::allocate() })
	}

	pub fn insert_flag(&mut self, flag: u32) {
		self.0.flags().insert(flag);
	}

	pub unsafe fn list_mut(&mut self) -> &mut GcMut<List> {
		self.0.gcmut()
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AnyValue {
		self.0.gcmut().as_mut_ptr()
	}

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flag(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.list_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity)).cast::<AnyValue>();
	}

	pub unsafe fn finish(self) -> Gc<List> {
		self.0.finish()
	}
}
