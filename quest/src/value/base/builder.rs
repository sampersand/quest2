use super::{Attribute, Base, Flags, Header, IntoParent};
use crate::value::gc::Gc;
use crate::Value;
use std::any::TypeId;
use std::ptr::{addr_of, addr_of_mut, NonNull};

/// The builder used for fine-grained control over making [`Base`]s.
///
/// If you don't need such precise control over [`Base`] creation, consider using [`Base::new`] or
/// [`Base::with_capacity`] instead.
///
/// # Example
/// ```
/// use quest::value::{ToValue, Gc};
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
/// let base = unsafe { builder.finish() };
/// let baseref = base.as_ref().expect("we hold the only reference");
///
/// assert_eq!(*baseref.data(), 0x1234);
/// assert_eq!(
///    "bar",
///    baseref.header()
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
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T> Builder<T> {
	/// Creates a new `Builder` and initializes the typeid.
	unsafe fn _new(ptr: NonNull<Base<T>>) -> Self {
		let mut builder = Self(ptr);

		// We have to use `addr_of_mut` in case the entire header wasn't zero-initialized,
		// as this function is also called from `new_uninit`.
		addr_of_mut!((*builder.base_mut()).header.typeid).write(TypeId::of::<T>());

		builder
	}

	/// Creates a new [`Builder`] from a pointer to a zero-initialized [`Base<T>`].
	///
	/// Note that [`allocate`] is preferred if you don't already have an allocated pointer, as it
	/// will zero allocate for you.
	///
	/// # Safety
	/// For this function to be safe to use, you must ensure the following invariants hold:
	/// - `ptr` was allocated via [`quest::alloc_zeroed`].
	/// - `ptr` is a properly aligned for `Base<T>`.
	/// - `ptr` can be written to.
	///
	/// *Technically*, `ptr` can be allocated via [`quest::alloc`]/[`quest::realloc`], however
	/// you would need to zero out the contents first. (At which point, just use
	/// [`quest::alloc_zeroed`].)
	///
	/// # Example
	/// ```
	/// # use quest::value::base::{Builder, Base};
	/// let layout = std::alloc::Layout::new::<Base<u8>>();
	///
	/// // SAFETY: we're guaranteed `layout` is not zero-sized,
	/// // as `Base<T>` is nonzero sized.
	/// let ptr = unsafe { quest::alloc_zeroed::<Base<u8>>(layout) };
	///
	/// // SAFETY: It was just zero allocated with the proper layout, which
	/// // also means we can write to it.
	/// let mut builder = unsafe { Builder::new_zeroed(ptr) };
	/// builder.set_data(12u8);
	///
	/// // SAFETY: Since it's zero-initialized, we only had to initialize the `data` field.
	/// let base = unsafe { builder.finish() };
	///
	/// assert_eq!(
	///     12u8,
	///     *base.as_ref().expect("we hold the only reference").data()
	/// );
	/// ```
	pub unsafe fn new_zeroed(ptr: NonNull<Base<T>>) -> Self {
		Self::_new(ptr)
	}

	/// Creates a new [`Builder`] from a pointer to an uninitialized [`Base<T>`].
	///
	/// Note that if `ptr` was allocated via [`quest::alloc_zeroed`], you should use [`new_zeroed`]
	/// instead, as it won't do unnecessary writes.
	///
	/// # Safety
	/// For this function to be safe to use, you must ensure the following invariants hold:
	/// - `ptr` was allocated via [`quest::alloc`]/[`quest::realloc`]/[`quest::alloc_zeroed`].
	/// - `ptr` is a properly aligned for `Base<T>`.
	/// - `ptr` can be written to.
	///
	/// # Example
	/// ```
	/// # use quest::value::base::{Builder, Base};
	/// let layout = std::alloc::Layout::new::<Base<u8>>();
	///
	/// // SAFETY: we're guaranteed `layout` is not zero-sized,
	/// // as `Base<T>` is nonzero sized.
	/// let ptr = unsafe { quest::alloc::<Base<u8>>(layout) };
	///
	/// // SAFETY: It was just allocated with the proper layout, which
	/// // also means we can write to it.
	/// let mut builder = unsafe { Builder::new_uninit(ptr) };
	///
	/// // As we didn't zero-initialize it, we need to call these three methods.
	/// builder.set_data(12u8);
	/// builder.allocate_attributes(0); // No attrs needed.
	/// builder.set_parents(quest::value::base::NoParents);
	///
	/// // SAFETY: We just initialized the data, attributes, and parents fields.
	/// let base = unsafe { builder.finish() };
	///
	/// assert_eq!(
	///     12u8,
	///     *base.as_ref().expect("we hold the only reference").data()
	/// );
	/// ```
	pub unsafe fn new_uninit(ptr: NonNull<Base<T>>) -> Self {
		let mut builder = Self::_new(ptr);

		// These fields would normally be zero-initialized, but as we cannot assume `ptr` was
		// zero-initialized, we have to do it ourselves
		addr_of_mut!((*builder.header_mut()).borrows).write(std::sync::atomic::AtomicU32::default());
		addr_of_mut!((*builder.header_mut()).flags).write(Flags::default());

		builder
	}

	/// Allocates a new, zero-initialized [`Base`].
	///
	/// This is a helper function which both zero allocates enough memory and constructs `Self`.
	///
	/// [`Base::builder`] is simply a convenience function for this.
	///
	/// # Example
	/// ```
	/// # use quest::value::base::Builder;
	/// let mut builder = Builder::<u8>::allocate();
	/// builder.set_data(34u8);
	///
	/// // SAFETY: Since it's zero-initialized, we only had to initialize the `data` field.
	/// let base = unsafe { builder.finish() };
	/// assert_eq!(
	///     34u8,
	///     *base.as_ref().expect("we hold the only reference").data()
	/// );
	/// ```
	pub fn allocate() -> Self {
		let layout = std::alloc::Layout::new::<Base<T>>();

		// SAFETY:
		// - For `alloc_zeroed`, we know `layout` is nonzero size, because `Base` alone is nonzero.
		// - For `new_uninit`, we know we can write to it and it is properly aligned because we just
		//   allocated it.
		unsafe { Self::new_zeroed(crate::alloc_zeroed(layout)) }
	}

	/// Gets a reference to the internal pointer.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::Base;
	/// let builder = Base::builder();
	/// let ptr = builder.as_ptr();
	/// // ... do stuff with `ptr`.
	/// # let _: std::ptr::NonNull<Base<i64>> = ptr;
	/// ```
	#[must_use]
	pub fn as_ptr(&self) -> NonNull<Base<T>> {
		self.0
	}

	/// Reserves enough space to store at least `attr_capacity` attributes within the `base`.
	///
	/// Note that while calling this function multiple times is not `unsafe`, it will leak memory.
	///
	/// # Examples
	/// ```
	/// # use quest::value::base::Base;
	/// use quest::value::{ToValue, Gc};
	/// use quest::value::ty::Text;
	///
	/// let mut builder = Base::<()>::builder();
	/// builder.allocate_attributes(3);
	///
	/// // SAFETY: we just allocated the attributes the line above.
	/// unsafe {
	///    builder.set_attr("foo".to_value(), "bar".to_value())?;
	///    builder.set_attr("baz".to_value(), "quux".to_value())?;
	///    builder.set_attr(12.to_value(), 34.to_value())?;
	/// }
	///
	/// // SAFETY: Since `builder` was zero-initialized to a ZST, we didn't have to do anything.
	/// let base = unsafe { builder.finish() };
	///
	/// assert_eq!(
	///    "bar",
	///     base.as_ref().expect("we hold the only reference")
	///         .header()
	///         .get_unbound_attr("foo".to_value())?
	///         .unwrap()
	///         .downcast::<Gc<Text>>()
	///         .unwrap()
	///         .as_ref()?
	///         .as_str()
	/// );
	/// # quest::Result::Ok(())
	/// ```
	pub fn allocate_attributes(&mut self, attr_capacity: usize) {
		unsafe { Header::attributes_raw_mut(self.header_mut()) }.allocate(attr_capacity);
	}

	/// Sets the parents for the base.
	///
	/// # Examples
	/// ```
	/// # use quest::value::base::Base;
	/// use quest::value::ty::{Kernel, Object, List, Singleton};
	///
	/// let parents = List::from_slice(&[
	///     Kernel::instance(),
	///     Object::instance(),
	/// ]);
	///
	/// let mut builder = Base::<()>::builder();
	/// builder.set_parents(parents);
	///
	/// // SAFETY: Since `builder` was zero-initialized to a ZST, we didn't have to do anything.
	/// let base = unsafe { builder.finish() };
	///
	/// assert!(
	///     base.as_mut().expect("we hold the only reference")
	///         .header_mut()
	///         .parents_mut()
	///         .as_list()
	///         .ptr_eq(parents)
	/// );
	/// ```
	pub fn set_parents<P: IntoParent>(&mut self, parent: P) {
		unsafe {
			Header::parents_raw_mut(self.header_mut()).set(parent);
		}
	}

	/// Access the flags in the header.
	///
	/// If you simply want to set flags, you can use the [`insert_user_flags`] shorthand. See [`Flags`]
	/// for more details on custom flags in general.
	///
	/// # Examples
	/// ```
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
	/// If you want to _access_ the flags, i.e. don't want to set them, then use [`flags`]. See
	/// [`Flags`] for more details on custom flags in general.
	///
	/// # Examples
	/// ```
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
	pub fn insert_user_flags(&self, flag: u32) {
		self.flags().insert_user(flag);
	}

	/// Assigns the data for the underlying `Base<T>`.
	///
	/// Unless `T` is a ZST (eg `()`), or `self` was zero-allocated and `T` has zero as a valid
	/// value, this function must be called before [`finish`] is called.
	///
	/// Note that calling this function multiple times in a row will simply overwrite the previous
	/// value, without running `T`'s destructor.
	///
	/// # Example
	/// ```
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
	pub fn set_data(&mut self, data: T) {
		// SAFETY: We know the pointer is aligned and can be written to b/c of Builder's invariants.
		unsafe {
			self.data_mut().write(data);
		}
	}

	/// Sets the attribute `attr` to `value` in the header.
	///
	/// If you know how many attributes you'll need beforehand, it's recommended that you call the
	/// [`allocate_attributes`] method first to ensure that you won't cause intermittent
	/// reallocations.
	///
	/// Note that this will overwrite any previous value associated with `attr` without warning.
	///
	/// # Safety
	/// If `self` was zero-initialized (eg via [`Base::builder`], [`allocate`], or [`new_zeroed`]),
	/// then there are no safety concerns. If you created `self` via [`new_uninit`] however, you must
	/// initialize the attributes field (eg via [`allocate_attributes`]) before calling this method.
	///
	/// # Example
	/// See [`allocate_attributes`] for examples.
	pub unsafe fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> crate::Result<()> {
		// SAFETY:
		// - `flags` is initialized in the constructors (`new_uninit` initializes to zero, and
		//   `new_zeroed` starts off with it at zero). The caller
		// - The caller guarantees that the attributes are initialized.
		Header::set_attr_raw(self.header_mut(), attr, value)
	}

	/// Get an immutable pointer to the underlying [`Base<T>`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::Base;
	/// let builder = Base::<i32>::builder();
	/// let base_ptr: *const Base<i32> = builder.base();
	/// // ... do stuff with the `base_ptr`.
	/// ```
	#[inline]
	#[must_use]
	pub fn base(&self) -> *const Base<T> {
		self.0.as_ptr()
	}

	/// Get a mutable pointer to the underlying [`Base<T>`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::Base;
	/// let mut builder = Base::<i32>::builder();
	/// let ptr: *mut Base<i32> = builder.base_mut();
	/// // ... do stuff with `ptr`.
	/// ```
	#[inline]
	#[must_use]
	pub fn base_mut(&mut self) -> *mut Base<T> {
		self.0.as_ptr()
	}

	/// Get an immutable pointer to the underlying [`Header`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::{Base, Header};
	/// let builder = Base::<i32>::builder();
	/// let ptr: *const Header = builder.header();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn header(&self) -> *const Header {
		// SAFETY: `self.base()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of!((*self.base()).header) }
	}

	/// Get a mutable pointer to the underlying [`Header`].
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::{Base, Header};
	/// let mut builder = Base::<i32>::builder();
	/// let ptr: *mut Header = builder.header_mut();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn header_mut(&mut self) -> *mut Header {
		// SAFETY: `self.base_mut()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of_mut!((*self.base_mut()).header) }
	}

	/// Get an immutable pointer to the underlying `T`.
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::Base;
	/// let builder = Base::<i32>::builder();
	/// let ptr: *const i32 = builder.data();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn data(&self) -> *const T {
		// SAFETY: `self.base()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of!((*self.base()).data).cast::<T>() }
	}

	/// Get a mutable pointer to the underlying `T`.
	///
	/// Note that the pointer may be pointing to uninitialized or invalid memory, as the underlying
	/// base may not have been fully initialized yet.
	///
	/// # Basic usage
	/// ```
	/// # use quest::value::base::Base;
	/// let mut builder = Base::<i32>::builder();
	/// let ptr: *mut i32 = builder.data_mut();
	/// // ... do stuff with `ptr`.
	/// ```
	#[must_use]
	pub fn data_mut(&mut self) -> *mut T {
		// SAFETY: `self.base_mut()` is a valid pointer to a `Base<T>`.
		unsafe { addr_of_mut!((*self.base_mut()).data).cast::<T>() }
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
	#[must_use]
	pub unsafe fn finish(self) -> Gc<Base<T>> {
		// SAFETY: The requirement for `new` was that the pointer was allocated via `crate::alloc` or
		// `crate::realloc`. Additionally, the caller ensures that the entire base was initialized.
		Gc::new(self.0)
	}
}
