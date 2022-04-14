pub use super::HasDefaultParent;
use crate::value::gc::Gc;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::AtomicU32; // pub is deprecated here, just to fix other things.

mod attributes;
mod builder;
mod flags;
mod parents;

pub use builder::Builder;
pub use flags::Flags;
pub use attributes::{AttributesGuard, Attribute};
pub use parents::{ParentsGuard, IntoParent, NoParents};

#[repr(C)]
pub struct Header {
	typeid: TypeId,
	parents: UnsafeCell<parents::Parents>,
	attributes: UnsafeCell<attributes::Attributes>,
	flags: Flags,
	borrows: AtomicU32,
}

sa::assert_eq_size!(Header, [u64; 4]);

#[repr(C, align(16))]
#[derive(Debug)]
pub struct Base<T: 'static> {
	header: Header,
	data: UnsafeCell<T>,
}

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
		*std::ptr::addr_of!((*this).header.typeid)
	}
}

impl<T> Base<T> {
	pub fn builder() -> Builder<T> {
		Builder::allocate()
	}

	pub fn new(data: T, parent: AnyValue) -> Gc<Self> {
		Self::new_with_capacity(data, parent, 0)
	}

	pub fn new_with_capacity(data: T, parent: AnyValue, attr_capacity: usize) -> Gc<Self> {
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
		self.header_data_mut().0
	}

	pub fn header_data_mut(&mut self) -> (&mut Header, &mut T) {
		(&mut self.header, unsafe { &mut *self.data.get() })
	}

	pub fn data(&self) -> &T {
		unsafe { &*self.data.get() }
	}

	pub fn data_mut(&mut self) -> &mut T {
		self.header_data_mut().1
	}
}

impl Drop for Header {
	fn drop(&mut self) {
		if let Ok(mut attrs) = self.attributes() {
			unsafe {
				attrs.drop_internal();
			}
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
	/// TODO: examples (happy path, try_hash failing, `gc<list>` mutably borrowed).
	pub fn get_unbound_attr<A: Attribute>(
		&self,
		attr: A,
		search_parents: bool,
	) -> Result<Option<AnyValue>> {
		if let Some(value) = self.attributes()?.get_unbound_attr(attr)? {
			return Ok(Some(value));
		}

		if search_parents {
			self.get_unbound_attr_from_parents(attr)
		} else {
			Ok(None)
		}
	}

	pub fn get_unbound_attr_from_parents<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		self.parents()?.get_unbound_attr(attr)
	}

	/// Gets the flags associated with the current object.
	// TODO: we need to somehow not expose the internal flags.
	pub fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Freezes the object, so that any future attempts to call [`Gc::as_mut`] will result in a
	/// [`Error::ValueFrozen`](crate::Error::ValueFrozen) being returned.
	///
	/// # Examples
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// let text = Text::from_static_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
	/// # qvm_rt::Result::<()>::Ok(())
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
		let attrs_ptr = (*std::ptr::addr_of!((*ptr).attributes)).get();
		let flags = &*std::ptr::addr_of!((*ptr).flags);
		AttributesGuard::new(attrs_ptr, flags)
			.map(|mut attrs| attrs.set_attr(attr, value))
			.ok_or_else(|| "attributes are already locked".to_string())?
	}

	/// Sets the `self`'s attribute `attr` to `value`.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, try_hash failing, `gc<list>` mutably borrowed).
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		// SAFETY: Since we're already initialized, all the safety concerns are fulfilled.
		unsafe { Self::set_attr_raw(self as *mut Self, attr, value) }
	}

	/// Attempts to delete `self`'s attribute `attr`, returning the old value if it was present.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, try_hash failing, `gc<list>` mutably borrowed).
	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		self.attributes()?.del_attr(attr)
	}

	pub fn parents(&self) -> Result<ParentsGuard<'_>> {
		unsafe { Self::parents_raw(self as *const Self) }
	}

	pub fn attributes(&self) -> Result<AttributesGuard<'_>> {
		unsafe { Self::attributes_raw(self as *const Self) }
	}

	pub unsafe fn parents_raw<'a>(ptr: *const Self) -> Result<ParentsGuard<'a>> {
		let parents_ptr = (*std::ptr::addr_of!((*ptr).parents)).get();
		let flags = &*std::ptr::addr_of!((*ptr).flags);

		ParentsGuard::new(parents_ptr, flags)
			.ok_or_else(|| "attributes are already locked".to_string().into())
	}

	pub unsafe fn attributes_raw<'a>(ptr: *const Self) -> Result<AttributesGuard<'a>> {
		let attrs_ptr = (*std::ptr::addr_of!((*ptr).attributes)).get();
		let flags = &*std::ptr::addr_of!((*ptr).flags);

		AttributesGuard::new(attrs_ptr, flags)
			.ok_or_else(|| "attributes are already locked".to_string().into())
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
