use super::{Base, Flags, Header};
use crate::value::base::{Attribute, AttributesMut, AttributesRef, ParentsMut, ParentsRef};
use crate::value::gc::{Allocated, Gc};
use crate::value::{AttributedMut, HasAttributes, HasParents};
use crate::{Result, Value};

use std::ptr::{addr_of, addr_of_mut, NonNull};

/// The builder used for fine-grained control over making [`Base`]s.
///
/// If you don't need such precise control over [`Base`] creation, consider using [`Base::new`] or
/// [`Base::new_with_capacity`] instead.
///
/// # Example
/// ```text
/// // this is `text` because i plan on overhauling the builder.
/// use quest::value::{ToValue, Gc, Attributed};
/// use quest::value::ty::{Pristine, Text};
/// use quest::value::base::{Base, Flags};
///
/// let mut builder = Base::<i64>::builder();
/// builder.set_data(0x1234);
/// builder.set_parents(Pristine::instance());
///
/// // Set attributes
/// builder.allocate_attributes(3);
///
/// // SAFETY: We just allocated the attributes.
/// unsafe {
///     builder.set_attr("foo".to_value(), "bar".to_value())?;
///     builder.set_attr("baz".to_value(), "quux".to_value())?;
///     builder.set_attr(12.to_value(), 34.to_value())?;
/// }
///
/// // You can also set custom flags, if you attribute meanings to them.
/// const FLAG_IS_SPECIAL: u32 = Flags::USER0;
/// builder.insert_user_flags(FLAG_IS_SPECIAL);
///
/// // SAFETY: Since `builder` was zero-initialized, we only had to set `data`.
/// let gc = unsafe { builder.finish() };
/// let gcref = gc.as_ref().expect("we hold the only reference");
///
/// assert_eq!(*gcref.data(), 0x1234);
/// assert_eq!(
///    "bar",
///    gcref
///        .get_unbound_attr("foo".to_value())?
///        .unwrap()
///        .downcast::<Gc<Text>>()
///        .unwrap()
///        .as_ref()?
///        .as_str()
/// );
/// # quest::Result::Ok(())
/// ```
#[must_use]
pub struct Builder<T: Allocated>(NonNull<Base<T>>);

impl<T: Allocated> Builder<T> {
	/// Allocates a new, zero-initialized [`Base`].
	///
	/// This is a helper function which both zero allocates enough memory and constructs `Self`.
	///
	/// [`Base::builder`] is simply a convenience function for this.
	///
	/// # Example
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Builder;
	/// let mut builder = Builder::<u8>::new();
	/// builder.set_data(34u8);
	///
	/// // SAFETY: Since it's zero-initialized, we only had to initialize the `data` field.
	/// let gc = unsafe { builder.finish() };
	/// assert_eq!(
	///     34u8,
	///     *gc.as_ref().expect("we hold the only reference").data()
	/// );
	/// ```
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	pub fn with_capacity(attr_capacity: usize) -> Self {
		let layout = std::alloc::Layout::new::<Base<T>>();

		// SAFETY:
		// - For `alloc_zeroed`, we know `layout` is nonzero size, because `Base` alone is nonzero.
		// - For `new_uninit`, we know we can write to it and it is properly aligned because we just
		//   allocated it.
		unsafe {
			let mut builder = Self(crate::alloc_zeroed(layout));

			builder.header_mut().flags = Flags::new(T::TYPE_FLAG as u32);
			builder.attributes_mut().allocate(attr_capacity);

			builder
		}
	}

	/// Gets a reference to the internal pointer.
	///
	/// # Basic usage
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Base;
	/// let builder = Base::builder();
	/// let ptr = builder.as_ptr();
	/// // ... do stuff with `ptr`.
	/// # let _: std::ptr::NonNull<Base<i64>> = ptr;
	/// ```
	#[must_use]
	pub(crate) fn as_ptr(&self) -> NonNull<Base<T>> {
		self.0
	}

	/// Access the flags in the header.
	///
	/// If you simply want to set flags, you can use the [`insert_user_flags`](
	/// Self::insert_user_flags) shorthand. See [`Flags`] for more details on custom flags in
	/// general.
	///
	/// # Examples
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::{Base, Flags};
	/// let mut builder = Base::<()>::builder();
	///
	/// const FLAG_IS_SUPER_DUPER_COOL: u32 = Flags::USER0;
	/// assert!(!builder.flags().contains(FLAG_IS_SUPER_DUPER_COOL));
	/// builder.insert_user_flags(FLAG_IS_SUPER_DUPER_COOL);
	///
	/// // SAFETY: Since `builder` was zero-initialized to a ZST, we didn't have to do anything.
	/// let base = unsafe { builder.finish() };
	///
	/// assert!(
	///     base.as_ref().expect("we hold the only reference")
	///         .flags()
	///         .contains(FLAG_IS_SUPER_DUPER_COOL)
	/// );
	/// ```
	#[must_use]
	pub fn flags(&self) -> &Flags {
		#[allow(clippy::deref_addrof)]
		// SAFETY: We know the pointer is aligned and can be read from b/c of Builder's invariants.
		unsafe {
			&*addr_of!((*self.header()).flags)
		}
	}

	/// Sets flags in the header's [`Flags`].
	///
	/// If you want to _access_ the flags, i.e. don't want to set them, then use [`flags`](
	/// Self::flags). See [`Flags`] for more details on custom flags in general.
	///
	/// # Examples
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::{Base, Flags};
	/// let mut builder = Base::<()>::builder();
	///
	/// // Set custom flags, if you attribute meanings to them.
	/// const FLAG_IS_SUPER_DUPER_COOL: u32 = Flags::USER0;
	/// builder.insert_user_flags(FLAG_IS_SUPER_DUPER_COOL);
	///
	/// // SAFETY: Since `builder` was zero-initialized to a ZST, we didn't have to do anything.
	/// let base = unsafe { builder.finish() };
	///
	/// assert!(
	///     base.as_ref().expect("we hold the only reference")
	///         .flags()
	///         .contains(FLAG_IS_SUPER_DUPER_COOL)
	/// );
	/// ```
	pub fn insert_user_flags(&self, flag: u32) {
		self.flags().insert_user(flag);
	}

	/// Assigns the data for the underlying `Base<T>`.
	///
	/// Unless `T` is a ZST (eg `()`), or `self` was zero-allocated and `T` has zero as a valid
	/// value, this function must be called before [`finish`](Self::finish) is called.
	///
	/// Note that calling this function multiple times in a row will simply overwrite the previous
	/// value, without running `T`'s destructor.
	///
	/// # Example
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::{Base, Flags};
	/// use std::num::NonZeroU64;
	/// let mut builder = Base::<NonZeroU64>::builder();
	///
	/// // Since `NonZeroU64` does not have zero as a valid variant,
	/// // we must call `set_data`:
	/// let twelve = NonZeroU64::new(12).expect("12 is not 0");
	/// builder.set_data(twelve);
	///
	/// // SAFETY: Since `builder` was zero-initialized, we only had to set the data.
	/// let base = unsafe { builder.finish() };
	///
	/// assert_eq!(
	///     twelve,
	///     *base.as_ref().expect("we hold the only reference")
	///         .data()
	/// );
	/// ```
	pub fn set_data(&mut self, data: T::Inner) {
		// SAFETY: We know the pointer is aligned and can be written to b/c of Builder's invariants.
		unsafe {
			self.data_mut().write(data);
		}
	}

	/// Get an immutable pointer to the underlying [`Base<T>`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Base;
	/// let builder = Base::<i32>::builder();
	/// let base_ptr: *const Base<i32> = builder.base();
	/// // ... do stuff with the `base_ptr`.
	/// ```
	#[inline]
	#[must_use]
	pub(crate) fn base(&self) -> *const Base<T> {
		self.0.as_ptr()
	}

	/// Get a mutable pointer to the underlying [`Base<T>`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Base;
	/// let mut builder = Base::<i32>::builder();
	/// let ptr: *mut Base<i32> = builder.base_mut();
	/// // ... do stuff with `ptr`.
	/// ```
	#[inline]
	#[must_use]
	pub(crate) fn base_mut(&mut self) -> *mut Base<T> {
		self.0.as_ptr()
	}

	fn header(&self) -> &Header {
		// SAFETY: `self.base()` is a valid pointer to a `Base<T>`.
		unsafe { &*addr_of!((*self.base()).header) }
	}

	fn header_mut(&mut self) -> &mut Header {
		// SAFETY: `self.base_mut()` is a valid pointer to a `Base<T>`.
		unsafe { &mut *addr_of_mut!((*self.base_mut()).header) }
	}

	/// Get an immutable pointer to the underlying `T`.
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Base;
	/// let builder = Base::<i32>::builder();
	/// let ptr: *const i32 = builder.data();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn data(&self) -> *const T::Inner {
		// SAFETY: `self.base()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of!((*self.base()).data).cast::<T::Inner>() }
	}

	/// Get a mutable pointer to the underlying `T`.
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```text
	/// // this is `text` because i plan on overhauling the builder.
	/// # use quest::value::base::Base;
	/// let mut builder = Base::<i32>::builder();
	/// let ptr: *mut i32 = builder.data_mut();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn data_mut(&mut self) -> *mut T::Inner {
		// SAFETY: `self.base_mut()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of_mut!((*self.base_mut()).data).cast::<T::Inner>() }
	}

	/// Finish building the `Base<T>` and return a garbage collected reference ([`Gc`]) to it.
	///
	/// # Safety
	/// To call this function, you must ensure that all fields have been initialized.
	///
	/// If `self` was created via [`new_uninit`], you must ensure that you've called
	/// [`set_parents`], [`allocate_attributes`], and [`set_data`].
	///
	/// If `self` was otherwise created (ie [`new_zeroed`] / [`allocate`] / [`Base::builder`]), you
	/// just need to call [`set_data`], and only if zero is an invalid representation of `T` (eg
	/// [`std::num::NonZeroU64`]).
	///
	/// Regardless of how `self` was created, if `T` is a ZST (eg `()`), then [`set_data`] does not
	/// need to be called.
	///
	/// # Examples
	/// See [`new_zeroed`] and [`new_uninit`] for examples.
	///
	/// [`new_uninit`]: Self::new_uninit
	/// [`set_parents`]: Self::set_parents
	/// [`allocate_attributes`]: Self::allocate_attributes
	/// [`set_data`]: Self::set_data
	/// [`new_zeroed`]: Self::new_zeroed
	/// [`allocate`]: Self::allocate
	#[must_use]
	pub unsafe fn finish(self) -> Gc<T> {
		// SAFETY: The requirement for `new` was that the pointer was allocated via `crate::alloc` or
		// `crate::realloc`. Additionally, the caller ensures that the entire base was initialized.
		//
		// lastly: This is valid, as `Allocated` guarantees that `T` and `Base<T>` are represented
		// identically, and thus converting a `Gc` of the two is valid.
		std::mem::transmute::<NonNull<Base<T>>, Gc<T>>(self.0)
	}
}

impl<T: Allocated> HasAttributes for Builder<T> {
	fn attributes(&self) -> AttributesRef<'_> {
		self.header().attributes()
	}

	fn attributes_mut(&mut self) -> AttributesMut<'_> {
		self.header_mut().attributes_mut()
	}
}

impl<T: Allocated> HasParents for Builder<T> {
	fn parents(&self) -> ParentsRef<'_> {
		self.header().parents()
	}

	fn parents_mut(&mut self) -> ParentsMut<'_> {
		self.header_mut().parents_mut()
	}
}

impl<T: Allocated> AttributedMut for Builder<T> {
	fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		self.attributes_mut().get_unbound_attr_mut(attr)
	}

	fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		self.attributes_mut().set_attr(attr, value)
	}

	fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		self.attributes_mut().del_attr(attr)
	}
}
