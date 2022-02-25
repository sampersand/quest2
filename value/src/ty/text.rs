use crate::base::Flags;
use crate::Gc;
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
	ptr: *mut u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedText {
	len: u8,
	buf: [u8; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedText>() - std::mem::size_of::<u8>();
const FLAG_EMBEDDED: u32 = Flags::USER1;
const FLAG_SHARED: u32 = Flags::USER2;
const FLAG_NOFREE: u32 = Flags::USER3;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<u8>(cap).unwrap()
}

impl Text {
	pub fn from_str(inp: &str) -> Gc<Self> {
		let mut builder = Self::builder(inp.len());
		builder.write(inp);
		builder.finish()
	}

	pub fn from_static_str(inp: &'static str) -> Gc<Self> {
		unsafe {
			let mut bb = crate::base::Base::<Text>::allocate();

			bb.flags().insert(FLAG_NOFREE);
			let mut data = bb.data_mut().assume_init_mut();
			data.alloc.ptr = inp.as_ptr() as *mut u8;
			data.alloc.len = inp.len();
			data.alloc.cap = data.alloc.len; // capacity = length

			bb.finish()
		}
	}

	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	pub fn with_capacity(cap: usize) -> Gc<Self> {
		// Nothing to do, as the default state is valid.
		Self::builder(cap).finish()
	}

	pub fn builder(cap: usize) -> Builder {
		Builder::new(cap)
	}

	fn has_flag(&self, flag: u32) -> bool {
		unsafe { Gc::from_ref(self) }.flags().contains(flag)
	}

	fn remove_flag(&self, flag: u32) {
		unsafe { Gc::from_ref(self) }.flags().remove(flag);
	}

	fn is_embedded(&self) -> bool {
		self.has_flag(FLAG_EMBEDDED)
	}

	fn is_shared(&self) -> bool {
		self.has_flag(FLAG_SHARED)
	}

	fn is_nofree(&self) -> bool {
		self.has_flag(FLAG_NOFREE)
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
	unsafe fn duplicate_alloc_ptr(&mut self, cap: usize) {
		debug_assert!(!self.is_embedded());

		let old_ptr = self.alloc.ptr;
		self.alloc.ptr = crate::alloc(alloc_ptr_layout(cap));
		self.alloc.cap = cap;
		std::ptr::copy(old_ptr, self.alloc.ptr, self.alloc.len);
	}

	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { &self.embed.buf }.as_ptr()
		} else {
			unsafe { self.alloc.ptr as *const u8 }
		}
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		if self.is_embedded() {
			return self.embed.buf.as_mut_ptr();
		}

		if self.is_nofree() || self.is_shared() {
			// Both static Rust strings (`FLAG_NOFREE`) and shared strings (`FLAG_SHARED`) don't allow
			// us to write to their pointer. As such, we need to duplicate the `alloc.ptr` field, which
			// gives us ownership of it. Afterwards, we have to remove the relevant flags.
			self.duplicate_alloc_ptr(self.alloc.len);
			self.remove_flag(FLAG_NOFREE | FLAG_SHARED);
		}

		self.alloc.ptr
	}

	pub fn as_bytes(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
	}

	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	#[inline]
	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe { std::str::from_utf8_unchecked_mut(self.as_mut_bytes()) }
	}

	pub fn clone(&self) -> Gc<Self> {
		if self.is_embedded() {
			return self.deep_clone();
		}

		unsafe {
			// For allocated strings, you can actually one-for-one copy the body,
			// as we now have `FLAG_SHARED` marked.
			Gc::from_ref(self).flags().insert(FLAG_SHARED);

			let mut bb = crate::base::Base::<Text>::allocate();
			bb.flags().insert(FLAG_SHARED);
			std::ptr::copy(self as *const Self, bb.data_mut().as_mut_ptr(), 1);
			bb.finish()
		}
	}

	pub fn deep_clone(&self) -> Gc<Self> {
		let mut builder = Self::builder(self.len());
		builder.write(self.as_str());
		builder.finish()
	}
}

impl Text {
	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.is_embedded());

		let mut new_cap = MAX_EMBEDDED_LEN * 2;
		if new_cap < required_len {
			new_cap = required_len;
		}

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embed.len as usize;
			let ptr = crate::alloc(layout);
			std::ptr::copy(self.embed.buf.as_ptr(), ptr, len);

			self.alloc = AllocatedText {
				len,
				cap: new_cap,
				ptr,
			};

			self.remove_flag(FLAG_EMBEDDED);
		}
	}

	fn allocate_more(&mut self, required_len: usize) {
		if self.is_embedded() {
			return self.allocate_more_embeded(required_len);
		}

		unsafe {
			self.alloc.cap *= 2;
			if self.alloc.cap < required_len {
				self.alloc.cap = required_len;
			}

			if self.is_nofree() {
				self.duplicate_alloc_ptr(required_len);
			} else {
				let layout = alloc_ptr_layout(self.alloc.cap);
				self.alloc.ptr = crate::realloc(self.alloc.ptr, layout, self.alloc.cap);
			}
		}
	}

	fn mut_end_ptr(&mut self) -> *mut u8 {
		unsafe { self.as_mut_ptr().offset(self.len() as isize) }
	}

	pub fn push(&mut self, chr: char) {
		let mut buf = [0u8; 4];
		chr.encode_utf8(&mut buf);

		let chrstr = unsafe { std::str::from_utf8_unchecked(&buf[..chr.len_utf8()]) };

		self.push_str(chrstr);
	}

	pub fn push_str(&mut self, string: &str) {
		if self.capacity() <= self.len() + string.len() {
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
		if self.is_embedded() || self.is_nofree() {
			return;
		}

		unsafe { alloc::dealloc(self.alloc.ptr, alloc_ptr_layout(self.alloc.cap)) }
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
	fn from(text: &str) -> Self {
		Text::from_str(text).into()
	}
}

impl crate::base::HasParents for Text {
	fn parents() -> crate::base::Parents {
		// TODO
		crate::base::Parents::NONE
	}
}
