use super::{Inner, Text, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::Builder as BaseBuilder;
use crate::value::gc::Gc;

#[must_use]
pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	pub unsafe fn new(ptr: std::ptr::NonNull<std::mem::MaybeUninit<Text>>) -> Self {
		Self(BaseBuilder::new(std::mem::transmute(ptr)))
	}

	pub fn allocate() -> Self {
		let alloc_ptr = BaseBuilder::<Inner>::allocate().inner_ptr();

		unsafe { Self::new(std::mem::transmute(alloc_ptr)) }
	}

	pub fn insert_flag(&mut self, flag: u32) {
		self.0.flags().insert(flag);
	}

	pub unsafe fn inner_mut(&mut self) -> &mut Inner {
		self.0.data_mut().assume_init_mut()
	}

	pub unsafe fn text_mut(&mut self) -> &mut Text {
		&mut *self.0.base_mut_ptr().cast::<Text>()
	}

	// unsafe because you should call this before you edit anything.
	pub unsafe fn allocate_buffer(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flag(FLAG_EMBEDDED);
			return;
		}

		let mut alloc = &mut self.inner_mut().alloc;

		// alloc.len is `0` because `Base::<T>::allocate` always zero allocates.
		alloc.cap = capacity;
		alloc.ptr = crate::alloc(super::alloc_ptr_layout(capacity));
	}

	#[must_use]
	pub unsafe fn finish(self) -> Gc<Text> {
		std::mem::transmute(self.0.finish())
	}
}
