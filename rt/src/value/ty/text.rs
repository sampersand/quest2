use crate::value::base::Flags;
use crate::value::gc::{Gc, GcRef, GcMut};
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

impl Gc<Text> {
	pub fn from_str(inp: &str) -> Self {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate(inp.len());
			builder.text_mut().push_str_unchecked(inp);
			builder.finish()
		}
	}

	pub fn from_static_str(inp: &'static str) -> Self {
		let mut builder = Self::builder();
		builder.insert_flag(FLAG_NOFREE);

		let mut alloc = unsafe { &mut builder.text_mut().alloc };
		alloc.ptr = inp.as_ptr() as *mut u8;
		alloc.len = inp.len();
		alloc.cap = alloc.len;

		unsafe { builder.finish() }
	}

	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate(capacity);
			builder.finish() // Nothing else to do, as the default state is valid.
		}
	}

	pub fn builder() -> Builder {
		Builder::new()
	}
}

impl GcRef<Text> {
	fn is_embedded(&self) -> bool {
		self.flags().contains(FLAG_EMBEDDED)
	}

	fn is_pointer_immutable(&self) -> bool {
		self.flags().contains_any(FLAG_NOFREE | FLAG_SHARED)
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

	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { &self.embed.buf }.as_ptr()
		} else {
			unsafe { self.alloc.ptr as *const u8 }
		}
	}

	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	pub fn clone(&self) -> Gc<Text> {
		if self.is_embedded() {
			// Since we're allocating a new `Self` anyways, we may as well copy over the data.
			return self.deep_clone();
		}

		unsafe {
			// For allocated strings, you can actually one-for-one copy the body, as we now
			// have `FLAG_SHARED` marked.
			self.flags().insert(FLAG_SHARED);

			let mut builder = Gc::<Text>::builder();
			std::ptr::copy(self.as_ptr(), (&mut **builder.text_mut() as *mut Text).cast::<u8>(), 1);
			builder.finish()
		}
	}

	pub fn deep_clone(&self) -> Gc<Text> {
		Gc::<Text>::from_str(self.as_str())
	}

	pub fn substr<I: std::slice::SliceIndex<str, Output=str>>(&self, idx: I) -> Gc<Text> {
		let slice = &self.as_str()[idx];

		unsafe {
			self.flags().insert(FLAG_SHARED);

			let mut builder = Gc::<Text>::builder();
			builder.insert_flag(FLAG_SHARED);
			builder.text_mut().alloc = AllocatedText {
				ptr: slice.as_ptr() as *mut u8,
				len: slice.len(),
				cap: slice.len(), // capacity = length
			};

			builder.finish()
		}
	}
}

impl GcMut<Text> {
	pub unsafe fn set_len(&mut self, new: usize) {
		if self.r().is_embedded() {
			assert!(new <= MAX_EMBEDDED_LEN);

			self.embed.len = new as u8;
		} else {
			self.alloc.len = new;
		}
	}

	unsafe fn duplicate_alloc_ptr(&mut self, capacity: usize) {
		debug_assert!(!self.r().is_embedded());

		let old_ptr = self.alloc.ptr;
		self.alloc.ptr = crate::alloc(alloc_ptr_layout(capacity));
		self.alloc.cap = capacity;
		std::ptr::copy(old_ptr, self.alloc.ptr, self.alloc.len);

		self.r().flags().remove(FLAG_NOFREE | FLAG_SHARED);
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		if self.r().is_embedded() {
			return self.embed.buf.as_mut_ptr();
		}

		if self.r().is_pointer_immutable() {
			// Both static Rust strings (`FLAG_NOFREE`) and shared strings (`FLAG_SHARED`) don't allow
			// us to write to their pointer. As such, we need to duplicate the `alloc.ptr` field, which
			// gives us ownership of it. Afterwards, we have to remove the relevant flags.
			self.duplicate_alloc_ptr(self.alloc.len);
		}

		self.alloc.ptr
	}
	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.r().len())
	}

	#[inline]
	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe { std::str::from_utf8_unchecked_mut(self.as_mut_bytes()) }
	}
}

impl GcMut<Text> {
	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.r().is_embedded());
		debug_assert!(required_len > MAX_EMBEDDED_LEN); // we should only every realloc at this point.

		let new_cap = std::cmp::max(MAX_EMBEDDED_LEN * 2, required_len);
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

			self.r().flags().remove(FLAG_EMBEDDED);
		}
	}

	fn allocate_more(&mut self, required_len: usize) {
		// If we're allocating more, and we're embedded, then we are going to need to allocate an
		// entirely new buffer in memory, and no longer be embedded.
		if self.r().is_embedded() {
			return self.allocate_more_embeded(required_len);
		}

		// Find the new capacity we'll need.
		let new_cap = std::cmp::max(unsafe { self.alloc.cap } * 2, required_len);

		// If the pointer is immutable, we have to allocate a new buffer, and then copy
		// over the data.
		if self.r().is_pointer_immutable() {
			unsafe { self.duplicate_alloc_ptr(new_cap); }
			return;
		}

		// We have unique ownership of our pointer, so we can `realloc` it without worry.
		unsafe {	
			let orig_layout = alloc_ptr_layout(self.alloc.cap);
			self.alloc.ptr = crate::realloc(self.alloc.ptr, orig_layout, new_cap);
			self.alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut u8 {
		unsafe { self.as_mut_ptr().offset(self.r().len() as isize) }
	}

	pub fn push(&mut self, chr: char) {
		let mut buf = [0u8; 4];
		chr.encode_utf8(&mut buf);

		let chrstr = unsafe { std::str::from_utf8_unchecked(&buf[..chr.len_utf8()]) };

		self.push_str(chrstr);
	}

	pub fn push_str(&mut self, string: &str) {
		if self.r().capacity() <= self.r().len() + string.len() {
			self.allocate_more(string.len());
		}

		unsafe {
			self.push_str_unchecked(string);
		}
	}

	pub unsafe fn push_str_unchecked(&mut self, string: &str) {
		debug_assert!(self.r().capacity() >= self.r().len() + string.len());

		std::ptr::copy(string.as_ptr(), self.mut_end_ptr(), string.len());
		self.set_len(self.r().len() + string.len())
	}

	// fn fix_idx(&self, idx: isize) -> Option<usize> {
	// 	if idx.is_positive() {
	// 		Some(idx as usize)
	// 	} else if self.len() as isize <= idx {
	// 		Some(self.len() - idx as usize)
	// 	} else {
	// 		None
	// 	}
	// }

	// pub fn substr2(&self, what: std::ops::Range<isize>) -> Gc<Self> {
	// 	let begin =
	// 		if let Some(idx) = self.fix_idx(what.start) {
	// 			idx
	// 		} else {
	// 			return Gc::default()
	// 		};

	// 	let stop = 
	// 		if let Some(idx) = self.fix_idx(what.end) {
	// 			idx
	// 		} else {
	// 			return Gc::default()
	// 		};

	// 	panic!()
	// }
}

impl Default for Gc<Text> {
	fn default() -> Self {
		Self::new()
	}
}

impl AsMut<str> for GcMut<Text> {
	fn as_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl AsRef<str> for GcRef<Text> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for GcMut<Text> {
	fn as_ref(&self) -> &str {
		self.r().as_ref()
	}
}

/*
impl Drop for Text {
	fn drop(&mut self) {
		if self.is_embedded() || self.is_nofree() || self.is_shared() {
			if self.is_shared() {
				todo!("we will just `return` normally, but ensure that the GC handles this case properly.");
			}

			return;
		}

		// FIXME: This will drop a pointer even if it is shared.
		// how do we want to deal with that, especially with substring shares, which dont
		// know where the entire string starts.

		unsafe { alloc::dealloc(self.alloc.ptr, alloc_ptr_layout(self.alloc.cap)) }
	}
}*/

impl Debug for GcRef<Text> {
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

impl Display for GcRef<Text> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl From<&'_ str> for Gc<Text> {
	fn from(string: &str) -> Self {
		Self::from_str(string)
	}
}

impl From<&'_ str> for crate::Value<Gc<Text>> {
	fn from(text: &str) -> Self {
		Gc::<Text>::from_str(text).into()
	}
}

impl crate::value::base::HasParents for Text {
	fn parents() -> crate::value::base::Parents {
		// TODO
		crate::value::base::Parents::NONE
	}
}

impl Eq for GcRef<Text> {}
impl PartialEq for GcRef<Text> {
	fn eq(&self, rhs: &Self) -> bool {
		self == rhs.as_str()
	}
}

impl PartialEq<str> for GcRef<Text> {
	fn eq(&self, rhs: &str) -> bool {
		self.as_str() == rhs
	}
}

impl PartialOrd for GcRef<Text> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for GcRef<Text> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

impl PartialOrd<str> for GcRef<Text> {
	fn partial_cmp(&self, rhs: &str) -> Option<std::cmp::Ordering> {
		self.as_str().partial_cmp(&rhs)
	}
}

#[cfg(test)]
mod tests {
	use crate::Value;
	use crate::value::Convertible;
	use super::*;
	use crate::value::ty::*;

	const JABBERWOCKY: &str = "twas brillig in the slithy tothe did gyre and gimble in the wabe";

	#[test]
	fn test_is_a() {
		assert!(<Gc<Text>>::is_a(Value::from("").any()));
		assert!(<Gc<Text>>::is_a(Value::from("x").any()));
		assert!(<Gc<Text>>::is_a(Value::from("yesseriie").any()));
		assert!(<Gc<Text>>::is_a(Value::from(JABBERWOCKY).any()));

		assert!(!<Gc<Text>>::is_a(Value::TRUE.any()));
		assert!(!<Gc<Text>>::is_a(Value::FALSE.any()));
		assert!(!<Gc<Text>>::is_a(Value::NULL.any()));
		assert!(!<Gc<Text>>::is_a(Value::ONE.any()));
		assert!(!<Gc<Text>>::is_a(Value::ZERO.any()));
		assert!(!<Gc<Text>>::is_a(Value::from(1.0).any()));
		assert!(!<Gc<Text>>::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(<Gc<Text>>::get(Value::from("")).as_ref().unwrap(), *"");
		assert_eq!(<Gc<Text>>::get(Value::from("x")).as_ref().unwrap(), *"x");
		assert_eq!(<Gc<Text>>::get(Value::from("yesseriie")).as_ref().unwrap(), *"yesseriie");
		assert_eq!(<Gc<Text>>::get(Value::from(JABBERWOCKY)).as_ref().unwrap(), *JABBERWOCKY);
	}
}
