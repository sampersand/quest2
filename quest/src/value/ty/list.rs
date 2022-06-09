use crate::value::base::Flags;
use crate::value::gc::{Allocated, Gc};
use crate::value::ty::{InstanceOf, Singleton};
use crate::Value;
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
	ptr: *mut Value,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedList {
	buf: [Value; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedList>() / std::mem::size_of::<Value>();
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

impl super::AttrConversionDefined for Gc<List> {
	const ATTR_NAME: crate::value::Intern = crate::value::Intern::to_list;
}

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<Value>(cap).unwrap()
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
	pub fn from_slice(inp: &[Value]) -> Gc<Self> {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate_buffer(inp.len());
			builder.list_mut().push_slice_unchecked(inp);
			builder.finish()
		}
	}

	#[must_use]
	pub fn from_static_slice(inp: &'static [Value]) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flags(FLAG_NOFREE | FLAG_SHARED);

		unsafe {
			let mut alloc = &mut builder.inner_mut().alloc;

			alloc.ptr = inp.as_ptr() as *mut Value;
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

	pub fn as_ptr(&self) -> *const Value {
		if self.is_embedded() {
			unsafe { &self.inner().embed.buf }.as_ptr()
		} else {
			unsafe { self.inner().alloc.ptr }
		}
	}

	#[inline]
	pub fn as_slice(&self) -> &[Value] {
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
	pub fn substr<I: std::slice::SliceIndex<[Value], Output = [Value]>>(&self, idx: I) -> Gc<Self> {
		let slice = &self.as_slice()[idx];

		unsafe {
			self.flags().insert_user(FLAG_SHARED);

			let mut builder = Self::builder();
			builder.insert_flags(FLAG_SHARED);
			builder.inner_mut().alloc = AllocatedList {
				ptr: slice.as_ptr() as *mut Value,
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

	pub unsafe fn as_mut_ptr(&mut self) -> *mut Value {
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

	pub fn as_mut_slice(&mut self) -> &mut [Value] {
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

			self.inner_mut().alloc = AllocatedList { len, cap: new_cap, ptr };

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
				new_cap * std::mem::size_of::<Value>(),
			)
			.as_ptr();

			alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut Value {
		unsafe { self.as_mut_ptr().add(self.len()) }
	}

	pub fn shift(&mut self) -> Option<Value> {
		let ret = self.as_slice().first().copied();

		if ret.is_some() {
			unsafe {
				self.set_len(self.len() - 1);
				self.as_mut_ptr().copy_from(self.as_mut_ptr().add(1), self.len());
			}
		}

		ret
	}

	pub fn unshift(&mut self, ele: Value) {
		if self.capacity() <= self.len() + 1 {
			self.allocate_more(1);
		}

		unsafe {
			self.as_mut_ptr().copy_to(self.as_mut_ptr().add(1), self.len());
			self.as_mut_ptr().write(ele);
			self.set_len(self.len() + 1);
		}
	}

	pub fn push(&mut self, ele: Value) {
		// OPTIMIZE: you can make this work better for single values.
		self.push_slice(std::slice::from_ref(&ele));
	}

	pub fn pop(&mut self) -> Option<Value> {
		let ret = self.as_slice().last().copied();

		if ret.is_some() {
			unsafe {
				self.set_len(self.len() - 1);
			}
		}

		ret
	}

	pub fn push_slice(&mut self, slice: &[Value]) {
		if self.capacity() <= self.len() + slice.len() {
			self.allocate_more(slice.len());
		}

		unsafe {
			self.push_slice_unchecked(slice);
		}
	}

	pub unsafe fn push_slice_unchecked(&mut self, slice: &[Value]) {
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

impl From<&'_ [Value]> for Gc<List> {
	fn from(string: &[Value]) -> Self {
		List::from_slice(string)
	}
}

impl From<&'_ [Value]> for crate::Value<Gc<List>> {
	fn from(text: &[Value]) -> Self {
		List::from_slice(text).into()
	}
}

pub mod funcs {
	use super::*;
	use crate::value::ty::Text;
	use crate::value::ToValue;
	use crate::{vm::Args, Result};

	pub fn len(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((list.as_ref()?.len() as i64).to_value())
	}

	pub fn eql(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if list.to_value().is_identical(args[0]) {
			return Ok(true.to_value());
		}

		let rhs = if let Some(rhs) = args[0].downcast::<Gc<List>>() {
			rhs
		} else {
			return Ok(false.to_value());
		};

		let lhsref = list.as_ref()?;
		let rhsref = rhs.as_ref()?;

		if lhsref.len() != rhsref.len() {
			return Ok(false.to_value());
		}

		for (&l, &r) in lhsref.as_slice().iter().zip(rhsref.as_slice()) {
			if !l.try_eq(r)? {
				return Ok(false.to_value());
			}
		}

		Ok(true.to_value())
	}

	pub fn index(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?; // todo: more positional args for slicing

		let listref = list.as_ref()?;
		let mut index = args[0].to_integer()?.get();

		if index < 0 {
			index += listref.len() as i64;

			if index < 0 {
				return Err("todo: error for out of bounds".to_string().into());
			}
		}

		Ok(*listref.as_slice().get(index as usize).expect("todo: error for out of bounds"))
	}

	pub fn index_assign(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?; // todo: more positional args for slicing

		let mut listmut = list.as_mut()?;
		let mut index = args[0].to_integer()?.get();
		let value = args[1];

		if index < 0 {
			index += listmut.len() as i64;

			if index < 0 {
				return Err("todo: error for out of bounds".to_string().into());
			}
		}

		assert!(index <= listmut.len() as _, "todo: index out of bounds fills with null");

		listmut.as_mut()[index as usize] = value;

		Ok(value)
	}

	pub fn push(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		list.as_mut()?.push(args[0]);

		Ok(list.to_value())
	}

	pub fn pop(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(list.as_mut()?.pop().unwrap_or_default())
	}

	pub fn shift(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(list.as_mut()?.shift().unwrap_or_default())
	}

	pub fn unshift(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?; // todo: more positional args for slicing

		list.as_mut()?.unshift(args[0]);

		Ok(list.to_value())
	}

	pub fn to_text(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		use crate::value::ty::text::SimpleBuilder;

		fn at_text_maybe_list(
			value: Value,
			builder: &mut SimpleBuilder,
			visited: &mut Vec<Gc<List>>,
		) -> Result<()> {
			if let Some(list) = value.downcast::<Gc<List>>() {
				at_text_recursive(list, builder, visited)
			} else {
				builder.push_str(value.dbg_text()?.as_ref()?.as_str());
				Ok(())
			}
		}

		fn at_text_recursive(
			list: Gc<List>,
			builder: &mut SimpleBuilder,
			visited: &mut Vec<Gc<List>>,
		) -> Result<()> {
			if visited.iter().any(|&ac| list.ptr_eq(ac)) {
				builder.push_str("[...]");
				return Ok(());
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
		Ok(builder.finish().to_value())
	}

	pub fn dbg(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		to_text(list, args)
	}

	pub fn map(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;
		let func = args[0];

		let listref = list.as_ref()?;
		let new = List::with_capacity(listref.len());
		let mut newmut = new.as_mut()?;

		for ele in listref.as_slice() {
			newmut.push(func.call(Args::new(&[*ele], &[]))?);
		}

		Ok(new.to_value())
	}

	pub fn each(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;
		let func = args[0];

		for ele in list.as_ref()?.as_slice() {
			func.call(Args::new(&[*ele], &[]))?;
		}

		Ok(list.to_value())
	}

	pub fn join(list: Gc<List>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.len() <= 1)?;

		let mut builder = Text::simple_builder();

		if let Some((first, rest)) = list.as_ref()?.as_slice().split_first() {
			builder.push_str(first.to_text()?.as_ref()?.as_str());

			let sep1 =
				if let Some(sep) = args.get(0).map(|x| x.try_downcast::<Gc<Text>>()).transpose()? {
					Some(sep.as_ref()?)
				} else {
					None
				};
			let sep = sep1.as_ref().map(|s| s.as_str()).unwrap_or_default();

			for ele in rest {
				builder.push_str(sep);
				builder.push_str(ele.to_text()?.as_ref()?.as_str());
			}
		}

		Ok(builder.finish().to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ListClass;

impl Singleton for ListClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "List", parent Object::instance();
				Intern::op_eql => method funcs::eql,
				Intern::op_index => method funcs::index,
				Intern::op_index_assign => method funcs::index_assign,
				Intern::len => method funcs::len,
				Intern::push => method funcs::push,
				Intern::pop => method funcs::pop,
				Intern::shift => method funcs::shift,
				Intern::unshift => method funcs::unshift,
				Intern::dbg => method funcs::dbg,
				Intern::to_text => method funcs::to_text,
				Intern::map => method funcs::map,
				Intern::each => method funcs::each,
				Intern::join => method funcs::join,
			}
		})
	}
}

impl InstanceOf for List {
	type Parent = ListClass;
}
