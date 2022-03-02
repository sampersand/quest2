use std::any::TypeId;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::atomic::AtomicU32;

mod attributes;
mod builder;
mod flags;

use attributes::Attributes;
pub use attributes::Parents;
pub use builder::Builder;
pub use flags::Flags;

pub trait HasParents {
	unsafe fn init();

	fn parents() -> Parents;
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Header {
	attributes: Attributes,
	pub(super) typeid: TypeId,
	flags: Flags,
	borrows: AtomicU32,
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Base<T: 'static> {
	pub(super) header: Header,
	data: UnsafeCell<MaybeUninit<T>>,
}

impl<T: HasParents> Base<T> {
	pub fn new(data: T) -> NonNull<Self> {
		unsafe {
			let mut builder = Self::allocate();
			builder.data_mut().write(data);
			builder.finish()
		}
	}

	pub unsafe fn allocate() -> Builder<T> {
		Self::allocate_with_parents(T::parents())
	}

	pub unsafe fn builder_inplace(base: NonNull<Self>) -> Builder<T> {
		let mut b = Builder::new(base);
		b._write_parents(T::parents());
		b
	}

	pub unsafe fn static_builder(base: &'static mut MaybeUninit<Self>) -> Builder<T> {
		let builder = Self::builder_inplace(NonNull::new_unchecked(base.as_mut_ptr()));
		builder.flags().insert(Flags::NOFREE);
		builder
	}
}

impl<T> Base<T> {
	pub unsafe fn allocate_with_parents(parents: Parents) -> Builder<T> {
		let mut b = Builder::allocate();
		b._write_parents(parents);
		b
	}

	pub const fn flags(&self) -> &Flags {
		&self.header.flags
	}

	pub const fn typeid(&self) -> TypeId {
		self.header.typeid
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
	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.attributes.get_attr(attr)
	}

	/// Gets the flags associated with the current object.
	// TODO: we need to somehow not expose the internal flags.
	pub const fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Freezes the object, so that any future attempts to call [`Gc::as_mut`] will result in a
	/// [`Error::ValueFrozen`] being returned.
	///
	/// # Examples
	/// ```rust
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// # pub fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
	/// # Ok(()) }
	/// ```
	pub fn freeze(&self) {
		self.flags().insert(Flags::FROZEN);
	}

	/// Gets a reference to the parents of this type.
	///
	/// Note that this is defined on [`GcMut`] and not [`GcRef`] because internally, not all parents
	/// are stored as a `Gc<List>`. When this function is called, the internal representation is set
	/// to a list, and then returned.
	///
	/// # Examples
	/// TODO: example
	pub fn parents(&mut self) -> crate::value::gc::Gc<crate::value::ty::List> {
		self.attributes.parents.as_list()
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
	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		self.attributes.set_attr(attr, value)
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
	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.attributes.del_attr(attr)
	}
}
