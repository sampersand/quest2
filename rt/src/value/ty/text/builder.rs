use super::{Text, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::Gc;
use crate::value::base::{Base, Builder as BaseBuilder};

pub struct Builder(BaseBuilder<Text>);

impl Builder {
	pub fn new() -> Self {
		Self(unsafe { Base::<Text>::allocate() })
	}

	pub fn insert_flag(&mut self, flag: u32) {
		self.0.flags().insert(flag);
	}

	pub unsafe fn text_mut(&mut self) -> &mut Text {
		self.0.data_mut().assume_init_mut()
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		self.text_mut().as_mut_ptr()
	}

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flag(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.text_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity));
	}

	pub unsafe fn finish(self) -> Gc<Text> {
		self.0.finish()
	}
}
