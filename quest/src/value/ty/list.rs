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
		self.0.data_mut()
	}

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
		alloc.ptr = crate::alloc(alloc_ptr_layout(capacity)).as_ptr();
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
		assert!(isize::try_from(new_cap).is_ok(), "too much memory allocated: {new_cap} bytes");

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embedded_len();
			let ptr = crate::alloc(layout).as_ptr();
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
		assert!(isize::try_from(new_cap).is_ok(), "too much memory allocated: {new_cap} bytes");

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
			.as_ptr();

			alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut AnyValue {
		unsafe { self.as_mut_ptr().add(self.len()) }
	}

	pub fn unshift(&mut self, ele: AnyValue) {
		if self.capacity() <= self.len() + 1 {
			self.allocate_more(1);
		}

		unsafe {
			self
				.as_mut_ptr()
				.copy_to(self.as_mut_ptr().add(1), self.len());
			self.as_mut_ptr().write(ele);
			self.set_len(self.len() + 1);
		}
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

quest_type_attrs! { for Gc<List>, parent Object;
	op_index => meth funcs::index,
	op_index_assign => meth funcs::index_assign,
	len => meth funcs::len,
	push => meth funcs::push,
	unshift => meth funcs::unshift,
	dbg => meth funcs::dbg,
	at_text => meth funcs::at_text,
	map => meth funcs::map,
	each => meth funcs::each,
}

pub mod funcs {
	use super::*;
	use crate::value::ty::{Integer, Text};
	use crate::value::ToAny;
	use crate::{vm::Args, Result};

	pub fn len(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok((list.as_ref()?.len() as Integer).to_any())
	}

	pub fn index(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?; // todo: more positional args for slicing

		let listref = list.as_ref()?;
		let mut index = args[0].convert::<Integer>()?;

		if index < 0 {
			index += listref.len() as Integer;

			if index < 0 {
				return Err("todo: error for out of bounds".to_string().into());
			}
		}

		Ok(*listref
			.as_slice()
			.get(index as usize)
			.expect("todo: error for out of bounds"))
	}

	pub fn index_assign(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?; // todo: more positional args for slicing

		let mut listmut = list.as_mut()?;
		let mut index = args[0].convert::<Integer>()?;
		let value = args[1];

		if index < 0 {
			index += listmut.len() as Integer;

			if index < 0 {
				return Err("todo: error for out of bounds".to_string().into());
			}
		}

		assert!(index <= listmut.len() as _, "todo: index out of bounds fills with null");

		listmut.as_mut()[index as usize] = value;

		Ok(value)
	}

	pub fn push(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?; // todo: more positional args for slicing

		list.as_mut()?.push(args[0]);

		Ok(list.to_any())
	}

	pub fn unshift(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?; // todo: more positional args for slicing

		list.as_mut()?.unshift(args[0]);

		Ok(list.to_any())
	}

	pub fn at_text(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		use crate::value::ty::text::SimpleBuilder;

		fn at_text_maybe_list(value: AnyValue, builder: &mut SimpleBuilder, visited: &mut Vec<Gc<List>>) -> Result<()> {
			if let Some(list) = value.downcast::<Gc<List>>() {
				at_text_recursive(list, builder, visited)
			} else {
				builder.push_str(&value.dbg_text()?.as_ref()?.as_str());
				Ok(())
			}
		}

		fn at_text_recursive(list: Gc<List>, builder: &mut SimpleBuilder, visited: &mut Vec<Gc<List>>) -> Result<()> {
			if visited.iter().any(|&ac| list.ptr_eq(ac)) {
				builder.push_str("[...]");
				return Ok(())
			}

			builder.push('[');
			if let Some((first, rest)) = list.as_ref()?.as_slice().split_first() {
				visited.push(list);
				at_text_maybe_list(*first, builder, visited)?;

				for element in rest {
					builder.push_str(", ");
					at_text_maybe_list(*element, builder, visited)?;
				}

				let last = visited.pop();
				debug_assert!(last.unwrap().ptr_eq(list));
			}
			builder.push(']');

			Ok(())
		}

		args.assert_no_arguments()?;

		let mut builder = Text::simple_builder();
		at_text_recursive(list, &mut builder, &mut Vec::new())?;
		Ok(builder.finish().to_any())
	}

	pub fn dbg(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		at_text(list, args)
	}

	pub fn map(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;
		let func = args[0];

		let listref = list.as_ref()?;
		let new = List::with_capacity(listref.len());
		let mut newmut = new.as_mut()?;

		for ele in listref.as_slice() {
			newmut.push(func.call(Args::new(&[*ele], &[]))?);
		}

		Ok(new.to_any())
	}

	pub fn each(list: Gc<List>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;
		let func = args[0];

		for ele in list.as_ref()?.as_slice() {
			func.call(Args::new(&[*ele], &[]))?;
		}

		Ok(list.to_any())
	}
}
