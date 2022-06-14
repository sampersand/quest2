//! Types relating to [`Base`], the type all allocated objects wrap.

pub use super::HasDefaultParent;
use crate::value::gc::{Allocated, Gc};
use crate::value::{Attributed, AttributedMut, HasAttributes, HasParents};
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::AtomicU32; // pub is deprecated here, just to fix other things.

mod attributes;
mod builder;
mod flags;
mod parents;

pub use attributes::{Attribute, AttributesMut, AttributesRef};
pub use builder::Builder;
pub use flags::{Flags, HasTypeFlag, TypeFlag};
pub use parents::{IntoParent, NoParents, ParentsMut, ParentsRef};

/// The header for allocated [`Value`]s.
///
/// All [allocated](crate::value::gc::Allocated) types in Quest internally begin with a
/// [`Header`]. This means that you can access the header for anything that's allocated without
/// actually knowing what type was allocated. Thus, you can, for example, lookup attributes on a
/// type without actually knowing what type it is.
#[repr(C)]
pub(super) struct Header {
	flags: Flags,
	borrows: AtomicU32,
	attributes: attributes::Attributes,
	parents: parents::Parents,
	_unused: [u8; 8],
}

sa::assert_eq_size!(Header, [u64; 4]);

/// The base for all allocated [`Value`]s.
///
/// All [allocated](crate::value::gc::Allocated) types in Quest are actually newtype wrappers
/// around a `Base<T>`. Thus, they all have a consistent layout, and begin with a header.
/// This allows for looking up attributes, parents, flags, etc. without having to downcast a
/// [`Gc<Any>`].
#[repr(C, align(16))]
pub(crate) struct Base<T: Allocated> {
	header: Header,
	data: T::Inner,
}

// TODO: are these actually safe? idts, since theyre wrapped in `Gc`
unsafe impl<T: Allocated> Send for Base<T> where T::Inner: Send + 'static {}
unsafe impl<T: Allocated> Sync for Base<T> where T::Inner: Sync + 'static {}

impl Debug for Header {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Header")
			.field("parents", &self.parents())
			.field("attributes", &self.attributes())
			.field("flags", &self.flags)
			.field("borrows", &self.borrows)
			.finish()
	}
}

impl<T: Allocated> Debug for Base<T>
where
	T::Inner: Debug,
{
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Base").field("header", &self.header).field("data", &self.data).finish()
		} else {
			Debug::fmt(&self.data, f)
		}
	}
}

impl Base<crate::value::value::Any> {
	pub(crate) unsafe fn _typeflag(this: *const Self) -> TypeFlag {
		(*this).header.flags.type_flag()
	}
}

impl<T: Allocated> Base<T> {
	/// Returns a new [`Builder`] for [`Base`]s.
	///
	/// This is a convenience method around [`Builder::allocate`]. Most of the time you won't need
	/// such fine control, and instead [`Base::new`]/[`Base::new_with_capacity`] can be used.
	pub fn builder() -> Builder<T> {
		Builder::allocate()
	}

	/// Creates a new [`Base`] with the given data and parents.
	#[must_use]
	pub fn new<P: IntoParent>(data: T::Inner, parent: P) -> Gc<T> {
		Self::new_with_capacity(data, parent, 0)
	}

	/// Creates a new [`Base`] with the given data, parents, and initial attribute capacity.
	#[must_use]
	pub fn new_with_capacity<P: IntoParent>(
		data: T::Inner,
		parent: P,
		attr_capacity: usize,
	) -> Gc<T> {
		let mut builder = Self::builder();

		builder.set_parents(parent);
		builder.set_data(data);
		builder.allocate_attributes(attr_capacity);

		unsafe { builder.finish() }
	}

	/// Gets a reference to the `data` for `self`.
	pub fn data(&self) -> &T::Inner {
		&self.data
	}

	/// Gets a mutable reference to the `data` for `self`.
	pub fn data_mut(&mut self) -> &mut T::Inner {
		&mut self.data
	}
}

impl Drop for Header {
	fn drop(&mut self) {
		unsafe {
			self.attributes_mut().drop_internal();
		}
	}
}

impl<T: Allocated> Drop for Base<T> {
	fn drop(&mut self) {
		// TODO: drop data.
	}
}

use crate::{value::Value, Result};

impl Header {
	/// Gets the borrows for `self`.
	pub(crate) fn borrows(&self) -> &AtomicU32 {
		&self.borrows
	}
	//
	//	/// Retrieves `self`'s attribute `attr`, returning `None` if it doesn't exist.
	//	///
	//	/// If `search_parents` is `false`, this function will only search the attributes defined
	//	/// directly on `self`. If `true`, it will also look through the parents for the attribute if it
	//	/// does not exist within our immediate attributes.
	//	///
	//	/// # Errors
	//	/// If the [`try_hash`](Value::try_hash) or [`try_eq`](Value::try_eq) functions on `attr`
	//	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	//	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	//	///
	//	/// # Example
	//	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	//	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
	//		self.get_unbound_attr_checked(attr, &mut Vec::new())
	//	}

	/// The same as [`get_bound_attr`](Self::get_unbound_attr), except with a list of values that
	/// have already been checked.
	///
	/// This function prevents duplicate checking of functions.
	pub fn get_unbound_attr_checked<A: Attribute>(
		&self,
		attr: A,
		checked: &mut Vec<Value>,
	) -> Result<Option<Value>> {
		if let Some(value) = self.attributes().get_unbound_attr(attr)? {
			Ok(Some(value))
		} else {
			self.parents().get_unbound_attr_checked(attr, checked)
		}
	}

	/// Gets mutable access to the attribute `attr`.
	///
	/// This doesn't have an "checked" variant, as only attributes are looked at.
	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		self.attributes_mut().get_unbound_attr_mut(attr)
	}

	/// Gets the flags associated with the current object.
	pub fn flags(&self) -> &Flags {
		&self.flags
	}

	/// Freezes the object, so that any future attempts to call [`Gc::as_mut`] will result in a
	/// [`ErrorKind::ValueFrozen`](crate::ErrorKind::ValueFrozen) being returned.
	///
	/// # Examples
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use quest::{ErrorKind, value::ty::Text};
	/// let text = Text::from_static_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert_matches!(text.as_mut().unwrap_err().kind, ErrorKind::ValueFrozen(_));
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
	unsafe fn set_attr_raw<A: Attribute>(ptr: *mut Self, attr: A, value: Value) -> Result<()> {
		let attrs_ptr = &mut (*ptr).attributes;
		let flags = &(*ptr).flags;

		attrs_ptr.guard_mut(flags).set_attr(attr, value)
	}

	/// Sets the `self`'s attribute `attr` to `value`.
	///
	/// # Errors
	/// If the [`try_hash`](Value::try_hash) or [`try_eq`](Value::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		if !attr.is_special() {
			return self.attributes_mut().set_attr(attr, value);
		}

		if attr.is_parents() {
			if let Some(list) = value.downcast::<Gc<crate::value::ty::List>>() {
				self.parents_mut().set(list);

				Ok(())
			} else {
				Err("can only set __parents__ to a List".to_string().into())
			}
		} else {
			unreachable!("unknown special attribute {attr:?}");
		}
	}

	/// Attempts to delete `self`'s attribute `attr`, returning the old value if it was present.
	///
	/// # Errors
	/// If the [`try_hash`](Value::try_hash) or [`try_eq`](Value::try_eq) functions on `attr`
	/// return an error, that will be propagated upwards. Additionally, if the parents of `self`
	/// are represented by a `Gc<List>`, which is currently mutably borrowed, this will also fail.
	///
	/// # Example
	/// TODO: examples (happy path, `try_hash` failing, `gc<list>` mutably borrowed).
	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		self.attributes_mut().del_attr(attr)
	}

	/// Gets an immutable reference to `self`'s parents.
	pub fn parents(&self) -> ParentsRef<'_> {
		unsafe { self.parents.guard_ref(&self.flags) }
	}

	/// Gets a mutable reference to `self`'s parents.
	pub fn parents_mut(&mut self) -> ParentsMut<'_> {
		unsafe { self.parents.guard_mut(&self.flags) }
	}

	/// Gets an immutable reference to `self`'s attributes.
	pub fn attributes(&self) -> AttributesRef<'_> {
		unsafe { self.attributes.guard_ref(&self.flags) }
	}

	/// Gets a mutable reference to `self`'s attributes.
	pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
		unsafe { self.attributes.guard_mut(&self.flags) }
	}

	/// <TODO: is this required?>
	unsafe fn parents_raw_mut<'a>(ptr: *mut Self) -> ParentsMut<'a> {
		let parents = &mut (*ptr).parents;
		let flags = &(*ptr).flags;

		parents.guard_mut(flags)
	}

	/// <TODO: is this required?>
	unsafe fn attributes_raw_mut<'a>(ptr: *mut Self) -> AttributesMut<'a> {
		let attrs_ptr = &mut (*ptr).attributes;
		let flags = &(*ptr).flags;

		attrs_ptr.guard_mut(flags)
	}
}

impl<T: Allocated> Base<T> {
	/// Gets a mutable reference to `self`'s attributes.
	pub fn _attributes_mut(&mut self) -> AttributesMut<'_> {
		unsafe { self.header.attributes.guard_mut(&self.header.flags) }
	}

	/// Gets a mutable reference to `self`'s parents.
	pub fn _parents_mut(&mut self) -> ParentsMut<'_> {
		unsafe { self.header.parents.guard_mut(&self.header.flags) }
	}

	pub fn deconstruct_mut(&mut self) -> (&mut T::Inner, AttributesMut<'_>, ParentsMut<'_>) {
		let attributes = unsafe { self.header.attributes.guard_mut(&self.header.flags) };
		let parents = unsafe { self.header.parents.guard_mut(&self.header.flags) };

		(&mut self.data, attributes, parents)
	}
}

impl<T: Allocated> Attributed for &Base<T> {
	fn get_unbound_attr_checked<A: Attribute>(
		self,
		attr: A,
		checked: &mut Vec<Value>,
	) -> Result<Option<Value>> {
		self.header.get_unbound_attr_checked(attr, checked)
	}
}

impl<T: Allocated> AttributedMut for Base<T> {
	fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		self.header.get_unbound_attr_mut(attr)
	}

	fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		self.header.set_attr(attr, value)
	}

	fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		self.header.del_attr(attr)
	}
}

impl<T: Allocated> HasParents for Base<T> {
	fn parents(&self) -> ParentsRef<'_> {
		self.header.parents()
	}

	fn parents_mut(&mut self) -> ParentsMut<'_> {
		self.header.parents_mut()
	}
}

impl<T: Allocated> HasAttributes for Base<T> {
	fn attributes(&self) -> AttributesRef<'_> {
		self.header.attributes()
	}

	fn attributes_mut(&mut self) -> AttributesMut<'_> {
		self.header.attributes_mut()
	}
}
