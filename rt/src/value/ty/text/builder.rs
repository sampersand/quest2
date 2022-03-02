use super::{Text, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::{Base, Builder as BaseBuilder};
use crate::value::gc::{Gc, GcMut};

pub struct Builder(BaseBuilder<Text>);

impl Builder {
	pub unsafe fn new(builder: BaseBuilder<Text>) -> Self {
		Self(builder)
	}

	pub fn allocate() -> Self {
		unsafe { Self::new(Base::<Text>::allocate()) }
	}

	pub fn insert_flag(&mut self, flag: u32) {
		self.0.flags().insert(flag);
	}

	pub unsafe fn text_mut(&mut self) -> &mut Text {
		self.0.data_mut().assume_init_mut()
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

		let mut alloc = &mut self.text_mut().0.data_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity));
	}

	pub unsafe fn finish(self) -> Gc<Text> {
		self.0.finish()
	}
}
