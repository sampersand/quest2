use crate::base::{Allocated, BaseFlags};
use std::alloc;
use crate::Gc;
use std::fmt::{self, Debug, Display, Formatter};

#[repr(C)]
pub union Text {
	alloc: AllocatedText,
	embed: EmbeddedText
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AllocatedText {
	len: usize,
	cap: usize,
	ptr: *mut u8
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedText {
	len: u8,
	buf: [u8; MAX_EMBEDDED_LEN]
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedText>() - std::mem::size_of::<u8>();
const FLAG_EMBEDDED: u32 = BaseFlags::USER1;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<u8>(cap).unwrap()
}

impl Text {
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	pub fn from_str(string: &str) -> Gc<Self> {
		let text = Self::with_capacity(string.len());
		unsafe {
			&mut (*text.as_mut_ptr_unchecked())
		}.push_str(string);
		text
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
				std::ptr::addr_of_mut!((*ptr).alloc.ptr).write(alloc::alloc(alloc_ptr_layout(cap)));
			}
		}

		base.inner()
	}

	pub unsafe fn set_len(&mut self, len: usize) {
		if self.is_embedded() {
			self.embed.len = len as u8;
		} else {
			self.alloc.len = len;
		}
	}

	pub fn resize(&mut self, capacity: usize) {
		let _ = capacity;
		// if capacity < MAX_EMBEDDED_LEN {
		// 	if self.is_embedded() {
		// 		unsafe {
		// 			self.set_len(capacity); // truncate it.
		// 		}
		// 		return; // you dont 
		// 	}
		// }
		// TODO

		// if capacity < MAX_EMBEDDED_LEN {
		// 	if self.is_embedded() {
		// 		return; // allocating
		// 	}
		// }
	}

	pub fn is_embedded(&self) -> bool {
		unsafe { &Gc::new(self.into()) }.flags().contains(FLAG_EMBEDDED)
	}

	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { self.embed.buf.as_ptr() }
		} else {
			unsafe { self.alloc.ptr as *const u8 }
		}
	}

	pub fn as_mut_ptr(&mut self) -> *mut u8 {
		self.as_ptr() as *mut u8
	}


	pub fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

	// safety: don't modify to make it an invalid `str`.
	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.as_bytes())
		}
	}

	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe {
			std::str::from_utf8_unchecked_mut(self.as_mut_bytes())
		}
	}

	pub fn len(&self) -> usize {
		if self.is_embedded() {
			unsafe { self.embed.len as usize }
		} else {
			unsafe { self.alloc.len }
		}
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

impl Text {
	fn allocate_more(&mut self, required_len: usize) {
		if !self.is_embedded() {
			let alloc = unsafe { &mut self.alloc };
			let orig_layout = alloc_ptr_layout(alloc.cap);

			alloc.cap *= 2;
			if alloc.cap < required_len {
				alloc.cap += required_len;
			}

			alloc.ptr = unsafe {
				alloc::realloc(alloc.ptr, orig_layout, alloc.cap)
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
			let ptr = alloc::alloc(layout);
			std::ptr::copy(self.embed.buf.as_ptr(), ptr, len);

			self.alloc = AllocatedText { len, cap: new_cap, ptr };

			(*Allocated::upcast_mut(self)).flags().remove(FLAG_EMBEDDED);
		}
	}

	fn mut_end_ptr(&mut self) -> *mut u8 {
		unsafe {
			self.as_mut_ptr().offset(self.len() as isize)
		}
	}

	pub fn push(&mut self, chr: char) {
		let mut buf = [0u8; 4];
		chr.encode_utf8(&mut buf);

		let chrstr = unsafe {
			std::str::from_utf8_unchecked(&buf[..chr.len_utf8()])
		};

		self.push_str(chrstr);
	}

	pub fn push_str(&mut self, string: &str) {
		if self.len() + string.len() > self.capacity() {
			self.allocate_more(string.len());
		}

		unsafe {
			std::ptr::copy(string.as_ptr(), self.mut_end_ptr(), string.len());

			self.set_len(self.len() + string.len())
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

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Text({:?})", self.as_str())
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl From<&'_ str> for Gc<Text> {
	fn from(string: &str) -> Self {
		Text::from_str(string)
	}
}

impl From<&'_ str> for crate::Value {
	fn from(string: &str) -> Self {
		Text::from_str(string).into()
	}
}
