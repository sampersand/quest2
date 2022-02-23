use super::{Text, MAX_EMBEDDED_LEN, FLAG_EMBEDDED};
use crate::base::{Base, Builder as BaseBuilder};

pub struct Builder {
	bb: BaseBuilder<Text>,
	ptr: *mut u8,
	cap: usize
}

impl Builder {
	pub fn new(cap: usize) -> Self {
		let mut this = Self {
			bb: unsafe { Base::<Text>::allocate() },
			ptr: std::ptr::null_mut(),
			cap
		};

		if cap <= MAX_EMBEDDED_LEN {
			this.bb.flags().insert(FLAG_EMBEDDED);
			unsafe {
				this.ptr = this.bb.data_mut().assume_init_mut().embed.buf.as_mut_ptr();
			}
		} else {
			unsafe {
				let mut alloc = &mut this.bb.data_mut().assume_init_mut().alloc;
				alloc.cap = cap;

				let layout = super::alloc_ptr_layout(cap);
				alloc.ptr = std::alloc::alloc(layout);
				this.ptr = alloc.ptr;

				if alloc.ptr.is_null() {
					std::alloc::handle_alloc_error(layout);
				}
			}
		}

		this
	}

	fn is_embedded(&self) -> bool {
		self.bb.flags().contains(FLAG_EMBEDDED)
	}

	unsafe fn increment_len_and_ptr(&mut self, len: usize) {
		self.ptr = self.ptr.offset(len as isize);

		if self.is_embedded() {
			assert!(len <= u8::MAX as usize, "len exceeds embedded length");
			self.bb.data_mut().assume_init_mut().embed.len += len as u8;
		} else {
			self.bb.data_mut().assume_init_mut().alloc.len += len;
		}
	}

	pub fn len(&self) -> usize {
		if self.is_embedded() {
			unsafe {
				self.bb.data().assume_init_ref().embed.len as usize
			}
		} else {
			unsafe {
				self.bb.data().assume_init_ref().alloc.len
			}
		}
	}

	pub fn cap(&self) -> usize {
		self.cap
	}

	pub fn write(&mut self, inp: &str) {
		assert!(inp.len() + self.len() <= self.cap(), "overflow initialization");

		unsafe {
			std::ptr::copy(inp.as_ptr(), self.ptr, inp.len());
			self.increment_len_and_ptr(inp.len());
		}
	}

	pub fn finish(self) -> crate::Gc<Text> {
		// We know this is safe, as any `unsafe` operations need to be completed correctly.
		unsafe {
			self.bb.finish()
		}
	}
}
