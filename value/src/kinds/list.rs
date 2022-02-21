use crate::base::{Allocated, BaseFlags};
use crate::Value;
use std::alloc;
use crate::Gc;
use std::fmt::{self, Debug, Formatter};

#[repr(C)]
pub union List {
	alloc: AllocatedList,
	embed: EmbeddedList
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AllocatedList {
	len: usize,
	cap: usize,
	ptr: *mut Value
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedList {
	len: usize, // might as well yknow.
	buf: [Value; MAX_EMBEDDED_LEN]
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedList>() - std::mem::size_of::<usize>();
const FLAG_EMBEDDED: u32 = BaseFlags::USER1;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<Value>(cap).unwrap()
}

impl List {
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	pub fn with_capacity(cap: usize) -> Gc<Self> {
		let base = unsafe {
			&mut *Allocated::<Self>::allocate()
		};

		if cap <= MAX_EMBEDDED_LEN {
			base.flags().insert(FLAG_EMBEDDED);
		} else {
			unsafe {
				let ptr = base.data_mut().as_mut_ptr();

				std::ptr::addr_of_mut!((*ptr).alloc.cap).write(cap);
				std::ptr::addr_of_mut!((*ptr).alloc.ptr).write(alloc::alloc(alloc_ptr_layout(cap)).cast());
			}
		}

		base.inner()
	}

	pub unsafe fn set_len(&mut self, len: usize) {
		if self.is_embedded() {
			self.embed.len = len;
		} else {
			self.alloc.len = len;
		}
	}

	pub fn is_embedded(&self) -> bool {
		unsafe { &Gc::new(self.into()) }.flags().contains(FLAG_EMBEDDED)
	}

	pub fn as_ptr(&self) -> *const Value {
		if self.is_embedded() {
			unsafe { self.embed.buf.as_ptr() as *const Value }
		} else {
			unsafe { self.alloc.ptr as *const Value }
		}
	}

	pub fn as_mut_ptr(&mut self) -> *mut Value {
		self.as_ptr() as *mut Value
	}

	pub fn as_slice(&self) -> &[Value] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

	pub fn as_mut_slice(&mut self) -> &mut [Value] {
		unsafe {
			std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
		}
	}

	pub fn len(&self) -> usize {
		unsafe { self.embed.len } // theyre both in the same position
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

impl List {
	fn allocate_more(&mut self, required_len: usize) {
		if !self.is_embedded() {
			let alloc = unsafe { &mut self.alloc };
			let orig_layout = alloc_ptr_layout(alloc.cap);

			alloc.cap *= 2;
			if alloc.cap < required_len {
				alloc.cap += required_len;
			}

			alloc.ptr = unsafe {
				alloc::realloc(alloc.ptr.cast(), orig_layout, alloc.cap).cast()
			};

			return;
		}

		let mut new_cap = MAX_EMBEDDED_LEN * 2;
		if new_cap < required_len {
			new_cap += required_len;
		}

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embed.len as usize;
			let ptr = alloc::alloc(layout).cast::<Value>();
			std::ptr::copy(self.embed.buf.as_ptr(), ptr, len);

			self.alloc = AllocatedList { len, cap: new_cap, ptr };

			(*Allocated::upcast_mut(self)).flags().remove(FLAG_EMBEDDED);
		}
	}

	fn mut_end_ptr(&mut self) -> *mut Value {
		unsafe {
			self.as_mut_ptr().offset(self.len() as isize)
		}
	}

	pub fn push(&mut self, value: Value) {
		if self.len() == self.capacity() {
			self.allocate_more(1);
		}

		unsafe {
			self.mut_end_ptr().write(value);
			self.set_len(self.len() + 1);
		}
	}

	pub fn extend_from_slice(&mut self, slice: &[Value]) {
		if self.len() + slice.len() > self.capacity() {
			self.allocate_more(slice.len());
		}

		unsafe {
			std::ptr::copy(slice.as_ptr(), self.mut_end_ptr(), slice.len());

			self.set_len(self.len() + slice.len())
		}
	}
}

impl AsRef<[Value]> for List {
	fn as_ref(&self) -> &[Value] {
		self.as_slice()
	}
}

impl AsMut<[Value]> for List {
	fn as_mut(&mut self) -> &mut [Value] {
		self.as_mut_slice()
	}
}

impl Drop for List {
	fn drop(&mut self) {
		if self.is_embedded() {
			return; // Nothing to deallocate
		}

		unsafe {
			alloc::dealloc(self.alloc.ptr.cast(), alloc_ptr_layout(self.alloc.cap))
		}
	}
}

impl Debug for List {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_tuple("List")
			.field(&self.as_slice())
			.finish()
	}
}
