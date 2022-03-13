use crate::value::gc::Gc;
use crate::value::ty::List;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::atomic::AtomicU32;

mod attributes;
mod builder;
mod flags;
mod parents;

pub use attributes::Attribute;
use attributes::Attributes;
pub use builder::Builder;
pub use flags::Flags;
pub use parents::Parents;

pub trait HasDefaultParent {
	fn parent() -> AnyValue;
}

#[repr(C)]
pub struct Header {
	pub(super) typeid: TypeId,
	parents: Parents,
	attributes: Attributes,
	flags: Flags,
	borrows: AtomicU32,
}

sa::assert_eq_align!(Header, u64);
sa::assert_eq_size!(Header, [u64; 4]);

#[derive(Debug)]
#[repr(C, align(16))]
pub struct Base<T: 'static> {
	pub(super) header: Header,
	data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + 'static> Send for Base<T> {}
unsafe impl<T: Sync + 'static> Sync for Base<T> {}

impl Debug for Header {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Header")
			.field("typeid", &self.typeid)
			.field("parents", &self.parents.debug(self.flags()))
			.field("attributes", &self.attributes.debug(self.flags()))
			.field("flags", &self.flags)
			.field("borrows", &self.borrows)
			.finish()
	}
}

impl<T: HasDefaultParent> Base<T> {
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
		b._write_parent(T::parent());
		b
	}

	pub unsafe fn static_builder(base: &'static mut MaybeUninit<Self>) -> Builder<T> {
		let builder = Self::builder_inplace(NonNull::new_unchecked(base.as_mut_ptr()));
		builder.flags().insert(Flags::NOFREE);
		builder
	}
}

impl<T> Base<T> {
	pub fn new_with_parent(data: T, parent: AnyValue) -> NonNull<Self> {
		unsafe {
			let mut builder = Self::allocate_with_parent(0, parent);
			builder.data_mut().write(data);
			builder.finish()
		}
	}

	pub unsafe fn allocate_with_parent(attr_capacity: usize, parent: AnyValue) -> Builder<T> {
		let mut b = Builder::allocate_with_capacity(attr_capacity);
		b._write_parent(parent);
		b
	}

	pub const fn header(&self) -> &Header {
		&self.header
	}

	pub fn header_mut(&mut self) -> &mut Header {
		&mut self.header
	}

	pub const fn data(&self) -> &T {
		unsafe { (*(self.data.get() as *const MaybeUninit<T>)).assume_init_ref() }
	}

	pub fn data_mut(&mut self) -> &mut T {
		unsafe { (*self.data.get()).assume_init_mut() }
	}
}

impl Drop for Header {
	fn drop(&mut self) {
		unsafe {
			Attributes::drop(&mut self.attributes, &self.flags);
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
	pub(crate) const fn borrows(&self) -> &AtomicU32 {
		&self.borrows
	}

	/// Retrieves `self`'s attribute `attr`, returning `None` if it doesn't exist.
	///
	/// # Errors
	/// If the [`try_hash`](AnyValue::try_hash) or [`try_eq`](AnyValue::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, try_hash failing, `gc<list>` mutably borrowed).
	pub fn get_unbound_attr<A: Attribute>(&self, attr: A, search_parents: bool) -> Result<Option<AnyValue>> {
		if let Some(value) = self.attributes.get_unbound_attr(attr, &self.flags)? {
			Ok(Some(value))
		} else if search_parents {
			self.parents.get_unbound_attr(attr, &self.flags)
		} else {
			Ok(None)
		}
	}

	/// Gets the flags associated with the current object.
	// TODO: we need to somehow not expose the internal flags.
	pub const fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Freezes the object, so that any future attempts to call [`Gc::as_mut`] will result in a
	/// [`Error::ValueFrozen`](crate::Error::ValueFrozen) being returned.
	///
	/// # Examples
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// let text = Text::from_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
	/// # qvm_rt::Result::<()>::Ok(())
	/// ```
	pub fn freeze(&self) {
		self.flags().insert(Flags::FROZEN);
	}

	/// Gets a reference to the parents of this type.
	///
	/// Note that this is mutable because, internally, not all parents are stored as a `Gc<List>`.
	/// When this function is called, the internal representation is set to a list, and then returned.
	///
	/// # Examples
	/// TODO: example
	pub fn parents_list(&mut self) -> Gc<List> {
		self.parents.as_list(&self.flags)
	}

	pub fn set_parents(&mut self, parents_list: Gc<List>) {
		self.parents.set_list(parents_list, &self.flags)
	}

	pub fn set_singular_parent(&mut self, parent: AnyValue) {
		self.parents.set_singular(parent, &self.flags)
	}

	pub(crate) fn parents(&self) -> Parents {
		self.parents
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
		self.attributes.set_attr(attr, value, &self.flags)
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
		self.attributes.del_attr(attr, &self.flags)
	}
}
