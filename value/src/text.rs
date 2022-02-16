use crate::base::{ValueBase, BaseFlags};
use std::alloc;
use crate::Gc;

#[repr(C)]
pub union Text {
	alloc: Allocated,
	embed: Embedded
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Allocated {
	len: usize,
	cap: usize,
	ptr: *mut u8
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Embedded {
	len: u8,
	buf: [u8; MAX_EMBEDDED_LEN]
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<Allocated>() - std::mem::size_of::<u8>();
const FLAG_EMBEDDED: BaseFlags = BaseFlags::USER1;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<u8>(cap).unwrap()
}

impl Text {
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	pub fn with_capacity(cap: usize) -> Gc<Self> {
		let base = unsafe { &mut *ValueBase::<Self>::allocate() };

		if cap <= MAX_EMBEDDED_LEN {
			base.flags_mut().insert(FLAG_EMBEDDED);
		} else {
			unsafe {
				let mut alloc = base.as_mut().alloc;

				alloc.cap = cap;
				alloc.ptr = alloc::alloc(alloc_ptr_layout(cap));
			}
		}

		base.inner()
	}

	pub fn resize(&mut self, capacity: usize) {
		// TODO
		
		// if capacity < MAX_EMBEDDED_LEN {
		// 	if self.is_embedded() {
		// 		return; // allocating
		// 	}
		// }
	}

	pub fn is_embedded(&self) -> bool {
		unsafe { ValueBase::upcast(self) }.flags().contains(FLAG_EMBEDDED)
	}

	pub fn as_bytes(&self) -> &[u8] {
		if self.is_embedded() {
			unsafe {
				std::slice::from_raw_parts(&self.embed.buf[0] as _, self.embed.len as usize)
			}
		} else {
			unsafe {
				std::slice::from_raw_parts(self.alloc.ptr, self.alloc.len)
			}
		}
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.as_bytes())
		}
	}

	// safety: don't modify to make it an invalid `str`.
	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		if self.is_embedded() {
			std::slice::from_raw_parts_mut(&mut self.embed.buf[0] as _, self.embed.len as usize)
		} else {
			std::slice::from_raw_parts_mut(self.alloc.ptr, self.alloc.len)
		}
	}

	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe {
			std::str::from_utf8_unchecked_mut(self.as_mut_bytes())
		}
	}

	pub fn len(&self) -> usize {
		self.as_bytes().len()
	}

	pub fn capacity(&self) -> usize {
		if self.is_embedded() {
			MAX_EMBEDDED_LEN
		} else {
			unsafe {
				self.alloc.cap
			}
		}
	}
}

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		if self.is_embedded() {
			return; // Nothing to deallocate
		}

		unsafe {
			alloc::dealloc(self.alloc.ptr, alloc_ptr_layout(self.alloc.cap))
		}
	}
}
