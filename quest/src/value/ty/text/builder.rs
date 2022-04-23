use super::{Inner, Text, FLAG_EMBEDDED, MAX_EMBEDDED_LEN};
use crate::value::base::Builder as BaseBuilder;
use crate::value::base::HasDefaultParent;
use crate::value::gc::Gc;
use std::ptr::NonNull;

#[must_use]
pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	/// Creates a new [`Builder`] from the given `ptr`.
	///
	/// # Safety
	/// - `ptr` must be properly aligned and writable.
	/// - `ptr` must be zero initialized.
	pub unsafe fn new(ptr: NonNull<Text>) -> Self {
		let mut builder = BaseBuilder::new_uninit(ptr.cast());

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

	// SAFETY: not actually unsafe, because `new` is the worrisome
	pub fn inner_mut(&mut self) -> &mut Inner {
		unsafe {
			&mut *self.0.data_mut()
		}
	}

	pub fn text_mut(&mut self) -> &mut Text {
		unsafe {
			&mut *self.0.base_mut().cast::<Text>()
		}
	}

	pub fn allocate_buffer(&mut self, capacity: usize) {
		if capacity <= MAX_EMBEDDED_LEN {
			self.insert_flags(FLAG_EMBEDDED);
			return;
		}


		unsafe {
			let ptr = crate::alloc(super::alloc_ptr_layout(capacity)).as_ptr();

			let mut alloc = &mut self.inner_mut().alloc;

			// since we're zero initialized, `alloc.len` is zero.
			alloc.cap = capacity;
			alloc.ptr = ptr;
		}
	}

	// not unsafe, as the default `allocate()` is safe, and `new` is unsafe.
	#[must_use]
	pub fn finish(mut self) -> Gc<Text> {
		self.text_mut().recalculate_hash(); // assign the hash.

		unsafe {
			Gc::from_inner(self.0.finish())
		}
	}
}