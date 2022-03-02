use crate::value::base::{Flags, Header};
use crate::value::gc::{Gc, GcMut, GcRef, Allocated};
use crate::value::AnyValue;
use std::alloc;
use std::fmt::{self, Debug, Formatter};

mod builder;
pub use builder::Builder;

impl Allocated for List {
	fn header(&self) -> &Header {
		todo!()
	}
	fn header_mut(&mut self) -> &mut Header {
		todo!()
	}
}

#[repr(C)]
pub union List {
	alloc: AllocatedList,
	embed: EmbeddedList,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AllocatedList {
	len: usize,
	cap: usize,
	ptr: *mut AnyValue,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedList {
	len: usize,
	buf: [AnyValue; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize =
	(std::mem::size_of::<AllocatedList>() - std::mem::size_of::<usize>()) / std::mem::size_of::<AnyValue>();

const FLAG_EMBEDDED: u32 = Flags::USER1;
const FLAG_SHARED: u32 = Flags::USER2;
const FLAG_NOFREE: u32 = Flags::USER3;

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<AnyValue>(cap).unwrap()
}

impl Gc<List> {
	pub fn from_slice(inp: &[AnyValue]) -> Self {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate(inp.len());
			builder.list_mut().push_slice_unchecked(inp);
			builder.finish()
		}
	}

	pub fn from_static_slice(inp: &'static [AnyValue]) -> Self {
		let mut builder = Self::builder();
		builder.insert_flag(FLAG_NOFREE);

		let mut alloc = unsafe { &mut builder.list_mut().alloc };
		alloc.ptr = inp.as_ptr() as *mut _;
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

impl Default for Gc<List> {
	fn default() -> Self {
		Self::new()
	}
}

impl GcRef<List> {
	fn is_embedded(&self) -> bool {
		self.flags().contains(FLAG_EMBEDDED)
	}

	fn is_pointer_immutable(&self) -> bool {
		self.flags().contains_any(FLAG_NOFREE | FLAG_SHARED)
	}

	pub fn len(&self) -> usize {
		// since both embedded and allocated have length at the same spot,
		// we can just pick one
		unsafe {
			self.embed.len 
		}
	}

	pub fn capacity(&self) -> usize {
		if self.is_embedded() {
			MAX_EMBEDDED_LEN
		} else {
			unsafe { self.alloc.cap }
		}
	}

	pub fn as_ptr(&self) -> *const AnyValue {
		if self.is_embedded() {
			unsafe { &self.embed.buf }.as_ptr()
		} else {
			unsafe { self.alloc.ptr as *const AnyValue }
		}
	}

	#[inline]
	pub fn as_slice(&self) -> &[AnyValue] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	pub fn clone(&self) -> Gc<List> {
		if self.is_embedded() {
			// Since we're allocating a new `Self` anyways, we may as well copy over the data.
			return self.deep_clone();
		}

		unsafe {
			// For allocated lists, you can actually one-for-one copy the body, as we now
			// have `FLAG_SHARED` marked.
			self.flags().insert(FLAG_SHARED);

			let mut builder = Gc::<List>::builder();
			let builder_ptr = (&mut **builder.list_mut() as *mut List).cast::<u8>();
			std::ptr::copy(self.as_ptr().cast::<u8>(), builder_ptr, 1);
			builder.finish()
		}
	}

	pub fn deep_clone(&self) -> Gc<List> {
		Gc::<List>::from_slice(self.as_slice())
	}

	pub fn sublist<I: std::slice::SliceIndex<[AnyValue], Output = [AnyValue]>>(&self, idx: I) -> Gc<List> {
		let slice = &self.as_slice()[idx];

		unsafe {
			self.flags().insert(FLAG_SHARED);

			let mut builder = Gc::<List>::builder();
			builder.insert_flag(FLAG_SHARED);
			builder.list_mut().alloc = AllocatedList {
				ptr: slice.as_ptr() as *mut _,
				len: slice.len(),
				cap: slice.len(), // capacity = length
			};

			builder.finish()
		}
	}
}

impl GcMut<List> {
	pub unsafe fn set_len(&mut self, new: usize) {
		if self.r().is_embedded() {
			assert!(new <= MAX_EMBEDDED_LEN);
		}

		// since `len` is in the same spot for both embed and alloc, we can do this.
		self.embed.len = new;
	}

	unsafe fn duplicate_alloc_ptr(&mut self, capacity: usize) {
		debug_assert!(!self.r().is_embedded());

		let old_ptr = self.alloc.ptr;
		self.alloc.ptr = crate::alloc(alloc_ptr_layout(capacity)).cast::<AnyValue>();
		self.alloc.cap = capacity;
		std::ptr::copy(old_ptr, self.alloc.ptr, self.alloc.len);

		self.r().flags().remove(FLAG_NOFREE | FLAG_SHARED);
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AnyValue {
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

	pub fn as_mut_slice(&mut self) -> &mut [AnyValue] {
		unsafe {
			std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.r().len())
		}
	}
}

impl GcMut<List> {
	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.r().is_embedded());
		debug_assert!(required_len > MAX_EMBEDDED_LEN); // we should only every realloc at this point.

		let new_cap = std::cmp::max(MAX_EMBEDDED_LEN * 2, required_len);
		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embed.len as usize;
			let ptr = crate::alloc(layout).cast::<AnyValue>();
			std::ptr::copy(self.embed.buf.as_ptr(), ptr, len);

			self.alloc = AllocatedList {
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
			unsafe {
				self.duplicate_alloc_ptr(new_cap);
			}
			return;
		}

		// We have unique ownership of our pointer, so we can `realloc` it without worry.
		unsafe {
			let orig_layout = alloc_ptr_layout(self.alloc.cap);
			self.alloc.ptr = crate::realloc(self.alloc.ptr.cast::<u8>(), orig_layout,
				new_cap * std::mem::size_of::<AnyValue>()).cast::<AnyValue>();
			self.alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut AnyValue {
		unsafe { self.as_mut_ptr().offset(self.r().len() as isize) }
	}

	pub fn push(&mut self, val: AnyValue) {
		// OPTIMIZE: you can make this work better for single values.
		self.push_slice(std::slice::from_ref(&val));
	}

	pub fn push_slice(&mut self, slice: &[AnyValue]) {
		if self.r().capacity() <= self.r().len() + slice.len() {
			self.allocate_more(slice.len());
		}

		unsafe {
			self.push_slice_unchecked(slice);
		}
	}

	pub unsafe fn push_slice_unchecked(&mut self, slice: &[AnyValue]) {
		debug_assert!(self.r().capacity() >= self.r().len() + slice.len());

		std::ptr::copy(slice.as_ptr(), self.mut_end_ptr(), slice.len());
		self.set_len(self.r().len() + slice.len())
	}
}

impl AsMut<[AnyValue]> for GcMut<List> {
	fn as_mut(&mut self) -> &mut [AnyValue] {
		self.as_mut_slice()
	}
}

impl AsRef<[AnyValue]> for GcRef<List> {
	fn as_ref(&self) -> &[AnyValue] {
		self.as_slice()
	}
}

impl AsRef<[AnyValue]> for GcMut<List> {
	fn as_ref(&self) -> &[AnyValue] {
		self.r().as_ref()
	}
}

/*
impl Drop for List {
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

impl Debug for GcRef<List> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.write_str("List(")?;
		}

		Debug::fmt(self.as_slice(), f)?;

		if f.alternate() {
			f.write_str(")")?;
		}

		Ok(())
	}
}

impl Debug for GcMut<List> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.r(), f)
	}
}

impl From<&'_ [AnyValue]> for Gc<List> {
	fn from(slice: &[AnyValue]) -> Self {
		Self::from_slice(slice)
	}
}

impl From<&'_ [AnyValue]> for crate::Value<Gc<List>> {
	fn from(slice: &[AnyValue]) -> Self {
		Gc::<List>::from_slice(slice).into()
	}
}

impl crate::value::base::HasParents for List {
	unsafe fn init() {
		// todo
	}

	fn parents() -> crate::value::base::Parents {
		Default::default() // todo
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use crate::value::ty::*;
// 	use crate::value::Convertible;
// 	use crate::Value;

// 	const JABBERWOCKY: &str = "twas brillig in the slithy tothe did gyre and gimble in the wabe";

// 	#[test]
// 	fn test_is_a() {
// 		assert!(<Gc<List>>::is_a(Value::from("").any()));
// 		assert!(<Gc<List>>::is_a(Value::from("x").any()));
// 		assert!(<Gc<List>>::is_a(Value::from("yesseriie").any()));
// 		assert!(<Gc<List>>::is_a(Value::from(JABBERWOCKY).any()));

// 		assert!(!<Gc<List>>::is_a(Value::TRUE.any()));
// 		assert!(!<Gc<List>>::is_a(Value::FALSE.any()));
// 		assert!(!<Gc<List>>::is_a(Value::NULL.any()));
// 		assert!(!<Gc<List>>::is_a(Value::ONE.any()));
// 		assert!(!<Gc<List>>::is_a(Value::ZERO.any()));
// 		assert!(!<Gc<List>>::is_a(Value::from(1.0).any()));
// 		assert!(!<Gc<List>>::is_a(Value::from(RustFn::NOOP).any()));
// 	}

// 	#[test]
// 	fn test_get() {
// 		assert_eq!(<Gc<List>>::get(Value::from("")).as_ref().unwrap(), *"");
// 		assert_eq!(<Gc<List>>::get(Value::from("x")).as_ref().unwrap(), *"x");
// 		assert_eq!(
// 			<Gc<List>>::get(Value::from("yesseriie")).as_ref().unwrap(),
// 			*"yesseriie"
// 		);
// 		assert_eq!(
// 			<Gc<List>>::get(Value::from(JABBERWOCKY)).as_ref().unwrap(),
// 			*JABBERWOCKY
// 		);
// 	}
// }
