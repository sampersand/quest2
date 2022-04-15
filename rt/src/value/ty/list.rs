use crate::value::base::Flags;
use crate::value::gc::{Allocated, Gc};
use crate::AnyValue;
use std::alloc;
use std::fmt::{self, Debug, Formatter};

mod builder;
pub use builder::Builder;

quest_type! {
	#[derive(NamedType)]
	pub struct List(Inner);
}

#[repr(C)]
#[doc(hidden)]
pub union Inner {
	// TODO: remove pub
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
	buf: [AnyValue; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize =
	std::mem::size_of::<AllocatedList>() / std::mem::size_of::<AnyValue>();
const FLAG_EMBEDDED: u32 = Flags::USER0;
const FLAG_SHARED: u32 = Flags::USER1;
const FLAG_NOFREE: u32 = Flags::USER2;
const EMBED_LENMASK: u32 = Flags::USER3 | Flags::USER4;

const _: () = assert!(MAX_EMBEDDED_LEN <= unmask_len(EMBED_LENMASK));

const fn unmask_len(len: u32) -> usize {
	debug_assert!(len & !EMBED_LENMASK == 0);
	(len >> 3) as usize
}

const fn mask_len(len: usize) -> u32 {
	debug_assert!(len <= MAX_EMBEDDED_LEN);
	(len as u32) << 3
}

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<AnyValue>(cap).unwrap()
}

impl List {
	fn inner(&self) -> &Inner {
		self.0.data()
	}

	fn inner_mut(&mut self) -> &mut Inner {
		self.0._data_mut()
	}

	#[must_use]
	pub fn builder() -> Builder {
		Builder::allocate()
	}

	#[must_use]
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	#[must_use]
	pub fn with_capacity(capacity: usize) -> Gc<Self> {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate_buffer(capacity);
			builder.finish() // Nothing else to do, as the default state is valid.
		}
	}

	#[must_use]
	pub fn from_slice(inp: &[AnyValue]) -> Gc<Self> {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate_buffer(inp.len());
			builder.list_mut().push_slice_unchecked(inp);
			builder.finish()
		}
	}

	#[must_use]
	pub fn from_static_slice(inp: &'static [AnyValue]) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flags(FLAG_NOFREE | FLAG_SHARED);

		unsafe {
			let mut alloc = &mut builder.inner_mut().alloc;

			alloc.ptr = inp.as_ptr() as *mut AnyValue;
			alloc.len = inp.len();
			alloc.cap = alloc.len;

			builder.finish()
		}
	}

	fn is_embedded(&self) -> bool {
		self.flags().contains(FLAG_EMBEDDED)
	}

	fn is_pointer_immutable(&self) -> bool {
		self.flags().contains_any(FLAG_NOFREE | FLAG_SHARED)
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> usize {
		if self.is_embedded() {
			self.embedded_len()
		} else {
			// SAFETY: we know we're allocated, as per the `if`.
			unsafe { self.inner().alloc.len }
		}
	}

	fn embedded_len(&self) -> usize {
		debug_assert!(self.is_embedded());
		unmask_len(self.flags().mask(EMBED_LENMASK))
	}

	pub unsafe fn set_len(&mut self, new_len: usize) {
		debug_assert!(new_len <= self.capacity(), "new len is larger than capacity");

		if self.is_embedded() {
			self.set_embedded_len(new_len);
		} else {
			self.inner_mut().alloc.len = new_len;
		}
	}

	fn set_embedded_len(&mut self, new_len: usize) {
		debug_assert!(self.is_embedded());

		self.flags().remove_user(EMBED_LENMASK);
		self.flags().insert_user(mask_len(new_len));
	}

	pub fn capacity(&self) -> usize {
		if self.is_embedded() {
			MAX_EMBEDDED_LEN
		} else {
			unsafe { self.inner().alloc.cap }
		}
	}

	pub fn as_ptr(&self) -> *const AnyValue {
		if self.is_embedded() {
			unsafe { &self.inner().embed.buf }.as_ptr()
		} else {
			unsafe { self.inner().alloc.ptr }
		}
	}

	#[inline]
	pub fn as_slice(&self) -> &[AnyValue] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	#[must_use]
	pub fn dup(&self) -> Gc<Self> {
		if self.is_embedded() {
			// Since we're allocating a new `Self` anyways, we may as well copy over the data.
			return self.deep_dup();
		}

		unsafe {
			// For allocated strings, you can actually one-for-one copy the body, as we now
			// have `FLAG_SHARED` marked.
			self.flags().insert_user(FLAG_SHARED);

			let mut builder = Self::builder();
			let builder_ptr = builder.inner_mut() as *mut Inner;
			builder_ptr.copy_from_nonoverlapping(self.inner() as *const Inner, 1);
			builder.finish()
		}
	}

	#[must_use]
	pub fn deep_dup(&self) -> Gc<Self> {
		Self::from_slice(self.as_slice())
	}

	#[must_use]
	pub fn substr<I: std::slice::SliceIndex<[AnyValue], Output = [AnyValue]>>(
		&self,
		idx: I,
	) -> Gc<Self> {
		let slice = &self.as_slice()[idx];

		unsafe {
			self.flags().insert_user(FLAG_SHARED);

			let mut builder = Self::builder();
			builder.insert_flags(FLAG_SHARED);
			builder.inner_mut().alloc = AllocatedList {
				ptr: slice.as_ptr() as *mut AnyValue,
				len: slice.len(),
				cap: slice.len(), // capacity = length
			};

			builder.finish()
		}
	}

	unsafe fn duplicate_alloc_ptr(&mut self, capacity: usize) {
		debug_assert!(self.is_pointer_immutable());

		let mut alloc = &mut self.inner_mut().alloc;
		let old_ptr = alloc.ptr;
		alloc.ptr = crate::alloc(alloc_ptr_layout(capacity))
			.as_ptr()
			.cast::<AnyValue>();
		alloc.cap = capacity;
		std::ptr::copy(old_ptr, alloc.ptr, alloc.len);

		self.flags().remove_user(FLAG_NOFREE | FLAG_SHARED);
	}

	pub unsafe fn as_mut_ptr(&mut self) -> *mut AnyValue {
		if self.is_embedded() {
			return self.inner_mut().embed.buf.as_mut_ptr();
		}

		if self.is_pointer_immutable() {
			// Both static Rust strings (`FLAG_NOFREE`) and shared strings (`FLAG_SHARED`) don't allow
			// us to write to their pointer. As such, we need to duplicate the `alloc.ptr` field, which
			// gives us ownership of it. Afterwards, we have to remove the relevant flags.
			self.duplicate_alloc_ptr(self.inner().alloc.len);
		}

		self.inner_mut().alloc.ptr
	}

	pub fn as_mut_slice(&mut self) -> &mut [AnyValue] {
		unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
	}

	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.is_embedded());

		let new_cap = std::cmp::max(MAX_EMBEDDED_LEN * 2, required_len);
		assert!(new_cap <= isize::MAX as usize, "too much memory allocated");

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embedded_len();
			let ptr = crate::alloc(layout).as_ptr().cast::<AnyValue>();
			std::ptr::copy(self.inner().embed.buf.as_ptr(), ptr, len);

			self.inner_mut().alloc = AllocatedList {
				len,
				cap: new_cap,
				ptr,
			};

			self.flags().remove_user(FLAG_EMBEDDED | EMBED_LENMASK);
		}
	}

	fn allocate_more(&mut self, required_len: usize) {
		// If we're allocating more, and we're embedded, then we are going to need to allocate an
		// entirely new buffer in memory, and no longer be embedded.
		if self.is_embedded() {
			return self.allocate_more_embeded(required_len);
		}

		// Find the new capacity we'll need.
		let new_cap = std::cmp::max(unsafe { self.inner().alloc.cap } * 2, required_len);
		assert!(new_cap <= isize::MAX as usize, "too much memory allocated");

		// If the pointer is immutable, we have to allocate a new buffer, and then copy
		// over the data.
		if self.is_pointer_immutable() {
			unsafe {
				self.duplicate_alloc_ptr(new_cap);
			}
			return;
		}

		// We have unique ownership of our pointer, so we can `realloc` it without worry.
		unsafe {
			let mut alloc = &mut self.inner_mut().alloc;

			alloc.ptr = crate::realloc(
				alloc.ptr.cast::<u8>(),
				alloc_ptr_layout(alloc.cap),
				new_cap * std::mem::size_of::<AnyValue>(),
			)
			.as_ptr()
			.cast::<AnyValue>();

			alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut AnyValue {
		unsafe { self.as_mut_ptr().add(self.len()) }
	}

	pub fn push(&mut self, ele: AnyValue) {
		// OPTIMIZE: you can make this work better for single values.
		self.push_slice(std::slice::from_ref(&ele));
	}

	pub fn push_slice(&mut self, slice: &[AnyValue]) {
		if self.capacity() <= self.len() + slice.len() {
			self.allocate_more(slice.len());
		}

		unsafe {
			self.push_slice_unchecked(slice);
		}
	}

	pub unsafe fn push_slice_unchecked(&mut self, slice: &[AnyValue]) {
		debug_assert!(self.capacity() >= self.len() + slice.len());

		std::ptr::copy(slice.as_ptr(), self.mut_end_ptr(), slice.len());
		self.set_len(self.len() + slice.len());
	}
}

impl Default for Gc<List> {
	fn default() -> Self {
		List::new()
	}
}

impl AsRef<[AnyValue]> for List {
	fn as_ref(&self) -> &[AnyValue] {
		self.as_slice()
	}
}

impl AsMut<[AnyValue]> for List {
	fn as_mut(&mut self) -> &mut [AnyValue] {
		self.as_mut_slice()
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

impl Debug for List {
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

impl From<&'_ [AnyValue]> for Gc<List> {
	fn from(string: &[AnyValue]) -> Self {
		List::from_slice(string)
	}
}

impl From<&'_ [AnyValue]> for crate::Value<Gc<List>> {
	fn from(text: &[AnyValue]) -> Self {
		List::from_slice(text).into()
	}
}

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct ListClass(());
}

singleton_object! { for ListClass, parentof List;
	// "+" => method!(qs_add),
	// "@text" => method!(qs_at_text)
}

// impl Eq for List {}
// impl PartialEq for List {
// 	fn eq(&self, rhs: &Self) -> bool {
// 		self == rhs.as_slice()
// 	}
// }

// impl PartialEq<[AnyValue]> for List {
// 	fn eq(&self, rhs: &[AnyValue]) -> bool {
// 		self.as_slice() == rhs
// 	}
// }

// impl PartialOrd for List {
// 	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
// 		Some(self.cmp(rhs))
// 	}
// }

// impl Ord for List {
// 	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
// 		self.as_str().cmp(rhs.as_str())
// 	}
// }

// impl PartialOrd<[AnyValue]> for List {
// 	fn partial_cmp(&self, rhs: &[AnyValue]) -> Option<std::cmp::Ordering> {
// 		self.as_str().partial_cmp(&rhs)
// 	}
// }

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
// 		assert!(!<Gc<List>>::is_a(Default::default()));
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
