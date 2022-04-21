use super::{Inner, Text, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::Builder as BaseBuilder;
use crate::value::base::HasDefaultParent;
use crate::value::gc::Gc;

#[must_use]
pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	pub unsafe fn new(ptr: std::ptr::NonNull<std::mem::MaybeUninit<Text>>) -> Self {
		let mut builder = BaseBuilder::new_uninit(std::mem::transmute(ptr));
		builder.set_parents(Gc::<Text>::parent());
		Self(builder)
	}

	pub fn allocate() -> Self {
		let alloc_ptr = BaseBuilder::<Inner>::allocate().as_ptr();

		unsafe { Self::new(std::mem::transmute(alloc_ptr)) }
	}

	pub fn insert_flags(&mut self, flag: u32) {
		self.0.insert_user_flags(flag);
	}

	pub unsafe fn inner_mut(&mut self) -> &mut Inner {
		&mut *self.0.data_mut()
	}

	pub unsafe fn text_mut(&mut self) -> &mut Text {
		&mut *self.0.base_mut().cast::<Text>()
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
	pub unsafe fn finish(mut self) -> Gc<Text> {
		self.text_mut().recalculate_hash(); // assign the hash.
		Gc::from_inner(self.0.finish())
	}
}
