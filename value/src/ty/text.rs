use crate::Gc;
use crate::base::Flags;
use std::alloc;
use std::fmt::{self, Debug, Display, Formatter};

mod builder;
pub use builder::Builder;

#[repr(C)]
pub union Text {
	alloc: AllocatedText,
	embed: EmbeddedText,
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
const FLAG_EMBEDDED: u32 = Flags::USER1;
const FLAG_SHARED: u32 = Flags::USER2;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<u8>(cap).unwrap()
}

impl Text {
	pub fn from_str(inp: &str) -> Gc<Self> {
		let mut builder = Self::builder(inp.len());
		builder.write(inp);
		builder.finish()
	}

	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	pub fn with_capacity(cap: usize) -> Gc<Self> {
		let builder = Self::builder(cap);

		// Nothing to do, as the default state is valid.
		builder.finish()
	}

	pub fn builder(cap: usize) -> Builder {
		Builder::new(cap)
	}

	pub fn is_embedded(&self) -> bool {
		unsafe { Gc::from_ref(self) }.flags().contains(FLAG_EMBEDDED)
	}

	pub fn is_shared(&self) -> bool {
		unsafe { Gc::from_ref(self) }.flags().contains(FLAG_SHARED)
	}

	pub unsafe fn set_len(&mut self, new: usize) {
		if self.is_embedded() {
			assert!(new <= MAX_EMBEDDED_LEN);
			self.embed.len = new as u8;
		} else {
			self.alloc.len = new;
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
			unsafe { self.alloc.cap }
		}
	}
}

impl Text {
	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { self.embed.buf.as_ptr() }
		} else {
			unsafe { self.alloc.ptr as *const u8 }
		}
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		if self.is_shared() {
			// Note that we don't `drop`, self, as we dont have unique control over it.
			std::ptr::copy(self.deep_clone().as_ptr(), self as *mut Self, 1);
			// FIXME
		}

		return self.as_ptr() as *mut u8
	}

	pub fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

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

	pub fn clone(&self) -> Gc<Self> {
		// if self.is_embedded() {
		// 	return self.deep_clone();
		// }
		
		let gc = unsafe { Gc::from_ref(self) };
		gc.flags().insert(FLAG_SHARED);
		gc
	}

	pub fn deep_clone(&self) -> Gc<Self> {
		let mut builder = Self::builder(self.len());
		builder.write(self.as_str());
		builder.finish()
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
			// TODO: check for return value

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

			Gc::from_ref(self).flags().remove(FLAG_EMBEDDED);
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
		if f.alternate() {
			f.write_str("Text(")?;
		}

		Debug::fmt(self.as_str(), f)?;

		if f.alternate() {
			f.write_str(")")?;
		}

		Ok(())
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

impl From<&'_ str> for crate::Value<Gc<Text>> {
	fn from(string: &str) -> Self {
		Text::from_str(string).into()
	}
}

impl crate::base::HasParents for Text {
	fn parents() -> crate::base::Parents {
		// TODO
		crate::base::Parents::NONE
	}
}

