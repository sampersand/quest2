pub use super::HasDefaultParent;
use crate::value::gc::Gc;
use std::any::TypeId;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::AtomicU32; // pub is deprecated here, just to fix other things.

mod attributes;
mod builder;
mod data;
mod flags;
mod parents;

pub use attributes::{Attribute, AttributesRef, AttributesMut};
pub use builder::Builder;
pub use data::{DataMutGuard, DataRefGuard};
pub use flags::Flags;
pub use parents::{IntoParent, NoParents, ParentsRef, ParentsMut};

#[repr(C)]
pub struct Header {
	flags: Flags,
	borrows: AtomicU32,
	attributes: attributes::Attributes,
	parents: parents::Parents,
	typeid: TypeId,
}

sa::assert_eq_size!(Header, [u64; 4]);

#[repr(C, align(16))]
pub struct Base<T: 'static> {
	header: Header,
	data: T,
}

// TODO: are these actually safe? idts, since theyre wrapped in `Gc`
unsafe impl<T: Send + 'static> Send for Base<T> {}
unsafe impl<T: Sync + 'static> Sync for Base<T> {}

impl Debug for Header {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		struct TypeIdDebug(TypeId);
		impl Debug for TypeIdDebug {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				write!(f, "{:?}", self.0)
			}
		}

		f.debug_struct("Header")
			.field("typeid", &TypeIdDebug(self.typeid))
			.field("parents", &self.parents())
			.field("attributes", &self.attributes())
			.field("flags", &self.flags)
			.field("borrows", &self.borrows)
			.finish()
	}
}

impl<T: Debug> Debug for Base<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.data, f)
	}
}

// impl<T> Base<T> {
// 	pub fn new<P: IntoParent>(data: T, parent: P) -> NonNull<Self> {
// 		unsafe {
// 			let mut builder = Self::allocate();
// 			builder.write_data(data);
// 			builder.set_parents(parent);
// 			builder.finish()
// 		}
// 	}

// 	pub unsafe fn allocate<P: IntoParent>(parent: P) -> Builder<T> {
// 		Self::allocate_with_capacity(0)
// 	}

// 	pub unsafe fn allocate_with_capacity(attr_capacity: usize) -> Builder<T> {
// 		Self::allocate_with_parent(attr_capacity, T::parent())
// 	}
// }

impl<T: HasDefaultParent> Base<T> {
	/*)
	/// Creates a new `Base<T>` with the given data, and its parents.
	pub fn new(data: T) -> NonNull<Self> {
		unsafe {
			let mut builder = Self::allocate();
			builder.data_mut().write(data);
			builder.finish()
		}
	}

	pub unsafe fn allocate() -> Builder<T> {
		Self::allocate_with_capacity(0)
	}

	pub unsafe fn allocate_with_capacity(attr_capacity: usize) -> Builder<T> {
		Self::allocate_with_parent(attr_capacity, T::parent())
	}

	pub unsafe fn builder_inplace(base: NonNull<Self>) -> Builder<T> {
		let mut b = Builder::new(base);
		b.set_parents(T::parent());
		b
	}

	pub unsafe fn static_builder(base: &'static mut MaybeUninit<Self>) -> Builder<T> {
		let builder = Self::builder_inplace(NonNull::new_unchecked(base.as_mut_ptr()));
		builder.flags().insert(Flags::NOFREE);
		builder
	}*/
}

impl Base<crate::value::value::Any> {
	pub(crate) unsafe fn _typeid(this: *const Self) -> TypeId {
		(*this).header.typeid
	}
}

impl<T> Base<T> {
	pub fn builder() -> Builder<T> {
		Builder::allocate()
	}

	pub fn new<P: IntoParent>(data: T, parent: P) -> Gc<Self> {
		Self::new_with_capacity(data, parent, 0)
	}

	pub fn new_with_capacity<P: IntoParent>(data: T, parent: P, attr_capacity: usize) -> Gc<Self> {
		let mut builder = Self::builder();

		builder.set_parents(parent);
		builder.set_data(data);
		builder.allocate_attributes(attr_capacity);

		unsafe { builder.finish() }
	}

	pub unsafe fn allocate_with_parent(attr_capacity: usize, parent: AnyValue) -> Builder<T> {
		let mut b = Builder::allocate();
		b.allocate_attributes(attr_capacity);
		b.set_parents(parent);
		b
	}

	pub fn header(&self) -> &Header {
		&self.header
	}

	pub fn header_mut(&mut self) -> &mut Header {
		&mut self.header
	}

	pub fn header_data_mut(&mut self) -> (&mut Header, &mut T) {
		(&mut self.header, &mut self.data)
	}

	pub fn data(&self) -> &T {
		&self.data
	}

	pub fn data_mut(&mut self) -> &mut T {
		&mut self.data
	}

	pub unsafe fn data_mut_raw<'a>(ptr: *mut Self) -> Result<DataMutGuard<'a, T>> {
		let data_ptr = &mut (*ptr).data;
		let flags = &(*ptr).header.flags;
		let borrows = &(*ptr).header.borrows;

		// TODO: you can currently make something froze whilst it's mutably borrowed, fix it.
		if flags.contains(Flags::FROZEN) {
			return Err(
				"todo: how do we want to return an error here"
					.to_string()
					.into(),
			);
			// return Err(Error::ValueFrozen(Gc::new(ptr).any()));
		}

		DataMutGuard::new(data_ptr, flags, borrows)
			.ok_or_else(|| "data is already locked".to_string().into())
	}

	pub unsafe fn data_ref_raw<'a>(ptr: *const Self) -> Result<DataRefGuard<'a, T>> {
		let data_ptr = &(*ptr).data;
		let flags = &(*ptr).header.flags;
		let borrows = &(*ptr).header.borrows;

		DataRefGuard::new(data_ptr, flags, borrows)
			.ok_or_else(|| "data is already locked".to_string().into())
	}
}

impl Drop for Header {
	fn drop(&mut self) {
		unsafe {
			self.attributes_mut().drop_internal();
		}
	}
}

impl<T> Drop for Base<T> {
	fn drop(&mut self) {
		// TODO: drop data.
	}
}

use crate::{value::AnyValue, Result};

impl Header {
	pub(crate) fn borrows(&self) -> &AtomicU32 {
		&self.borrows
	}

	/// Retrieves `self`'s attribute `attr`, returning `None` if it doesn't exist.
	///
	/// If `search_parents` is `false`, this function will only search the attributes defined
	/// directly on `self`. If `true`, it will also look through the parents for the attribute if it
	/// does not exist within our immediate attributes.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	pub fn get_unbound_attr<A: Attribute>(
		&self,
		attr: A,
		search_parents: bool,
	) -> Result<Option<AnyValue>> {
		if let Some(value) = self.attributes().get_unbound_attr(attr)? {
			return Ok(Some(value));
		}

		if search_parents {
			self.get_unbound_attr_from_parents(attr)
		} else {
			Ok(None)
		}
	}

	pub fn get_unbound_attr_from_parents<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		self.parents().get_unbound_attr(attr)
	}

	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut AnyValue> {
		self.attributes_mut().get_unbound_attr_mut(attr)
	}

	/// Gets the flags associated with the current object.
	// TODO: we need to somehow not expose the internal flags.
	pub fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Freezes the object, so that any future attempts to call [`Gc::as_mut`] will result in a
	/// [`ErrorKind::ValueFrozen`](crate::error::ErrorKind::ValueFrozen) being returned.
	///
	/// # Examples
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use quest::{error::ErrorKind, value::ty::Text};
	/// let text = Text::from_static_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert_matches!(text.as_mut().unwrap_err().kind(), ErrorKind::ValueFrozen(_));
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn freeze(&self) {
		self.flags().insert_internal(Flags::FROZEN);
	}

	/// Sets the the attribute, but on a possibly-uninitialized `ptr`.
	///
	/// # Safety
	/// - `ptr` must be a valid pointer to a `Self` for read & writes
	/// - The `attribute`s field must have been initialized.
	/// - The `flags` field must have been initialized.
	unsafe fn set_attr_raw<A: Attribute>(ptr: *mut Self, attr: A, value: AnyValue) -> Result<()> {
		let attrs_ptr = &mut (*ptr).attributes;
		let flags = &(*ptr).flags;

		attrs_ptr.guard_mut(flags).set_attr(attr, value)
	}

	/// Sets the `self`'s attribute `attr` to `value`.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		self.attributes_mut().set_attr(attr, value)
	}

	/// Attempts to delete `self`'s attribute `attr`, returning the old value if it was present.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		self.attributes_mut().del_attr(attr)
	}

	pub fn parents(&self) -> ParentsRef<'_> {
		unsafe { self.parents.guard_ref(&self.flags) }
	}

	pub fn parents_mut(&mut self) -> ParentsMut<'_> {
		unsafe { self.parents.guard_mut(&self.flags) }
	}

	pub fn attributes(&self) -> AttributesRef<'_> {
		unsafe { self.attributes.guard_ref(&self.flags) }
	}

	pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
		unsafe { self.attributes.guard_mut(&self.flags) }
	}

	pub unsafe fn parents_raw<'a>(ptr: *const Self) -> ParentsRef<'a> {
		let parents = &(*ptr).parents;
		let flags = &(*ptr).flags;

		parents.guard_ref(flags)
	}

	pub unsafe fn parents_raw_mut<'a>(ptr: *mut Self) -> ParentsMut<'a> {
		let parents = &mut (*ptr).parents;
		let flags = &(*ptr).flags;

		parents.guard_mut(flags)
	}

	pub unsafe fn attributes_raw_mut<'a>(ptr: *mut Self) -> AttributesMut<'a> {
		let attrs_ptr = &mut (*ptr).attributes;
		let flags = &(*ptr).flags;

		attrs_ptr.guard_mut(flags)
	}
}

unsafe impl<T: 'static> super::gc::Allocated for Base<T> {
	type Inner = T;

	fn header(&self) -> &Header {
		&self.header
	}

	fn header_mut(&mut self) -> &mut Header {
		&mut self.header
	}

	fn flags(&self) -> &Flags {
		&self.header.flags
	}
}
