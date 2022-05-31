//! Types related to allocated Quest types.

use crate::value::base::{Attribute, Base, Flags, Header, IntoParent, ParentsRef, ParentsMut, AttributesRef, AttributesMut};
use crate::value::ty::{List, Wrap};
use crate::value::{value::Any, AnyValue, Convertible, ToAny, Value};
use crate::Result;
use std::fmt::{self, Debug, Formatter, Pointer};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};

/// A trait that indicates a type contains at a minimum a [`Header`].
///
/// # Safety
/// To safely implement this trait, you must guarantee that your type is a `#[repr(transparent)]`
/// wrapper around a `Base<T::Inner>`. That is, your struct must look like this:
/// ```no_run
/// use quest::value::{gc::Allocated, base::Base};
///
/// #[repr(transparent)]
/// struct MyStruct(Base<Inner>);
///
/// struct Inner { /* ... */ }
///
/// unsafe impl Allocated for MyStruct {
///     type Inner = Inner;
/// }
/// ```
///
/// # See Also
/// - [`quest_type`] A macro that's used to create allocated types.
pub unsafe trait Allocated: 'static {
	#[doc(hidden)]
	type Inner;

	fn header(&self) -> &Header {
		unsafe { &*(self as *const Self).cast::<Header>() }
	}

	fn header_mut(&mut self) -> &mut Header {
		unsafe { &mut *(self as *mut Self).cast::<Header>() }
	}

	fn flags(&self) -> &Flags {
		self.header().flags()
	}
}

/// A garbage collected pointer to `T`.
///
/// All non-immediate types in Quest are allocated on the heap. These types can never be accessed
/// directly, but must be interacted with through [`Gc`] or its references ([`Ref`] and [`Mut`]).
///
/// # Examples
/// ```
/// # use quest::value::{gc::{Gc, Ref, Mut}, ty::Text};
/// let text = Text::from_static_str("Quest is cool");
///
/// let textref: Ref<Text> = text.as_ref()?;
/// assert_eq!(*textref, "Quest is cool");
///
/// drop(textref);
/// let mut textmut: Mut<Text> = text.as_mut()?;
/// textmut.push('!');
///
/// assert_eq!(*textmut, "Quest is cool!");
/// # quest::Result::<()>::Ok(())
/// ```
#[repr(transparent)]
pub struct Gc<T: Allocated>(NonNull<T>);

unsafe impl<T: Allocated + Send> Send for Gc<T> {}
unsafe impl<T: Allocated + Sync> Sync for Gc<T> {}

impl<T: Allocated> Copy for Gc<T> {}
impl<T: Allocated> Clone for Gc<T> {
	fn clone(&self) -> Self {
		*self
	}
}

/// A trait implemented by types which have subvalues they must mark.
pub trait Mark {
	/// Mark the subvalues.
	fn mark(&self);
}

impl<T: Allocated> Debug for Gc<T>
where
	Ref<T>: Debug,
{
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if !f.alternate() {
			if let Ok(inner) = self.as_ref() {
				return Debug::fmt(&inner, f);
			}
		}

		write!(f, "Gc({:p}:", self.0)?;

		if let Ok(inner) = self.as_ref() {
			Debug::fmt(&inner, f)?;
		} else {
			write!(f, "<locked>")?;
		}

		write!(f, ")")
	}
}

impl<T: Allocated> Pointer for Gc<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

/*
impl<T: HasDefaultParent + Allocated> Gc<T> {
	/// Helper function for `Base::allocate`. See it for safety.
	pub(crate) unsafe fn allocate() -> Builder<T> {
		Base::allocate()
	}
*/

/// Sentinel value used to indicate the `Gc<T>` is mutably borrowed.
const MUT_BORROW: u32 = u32::MAX;
pub const MAX_BORROWS: usize = (MUT_BORROW - 1) as usize;

impl<T: Allocated> Gc<T> {
	/// Creates a new `Gc<T>` from `ptr`.
	///
	/// # Safety
	/// The `Base<T>` must have been allocated via [`quest::alloc`]/[`quest::alloc_zeroed`]/
	/// [`quest::realloc`]. Additionally, the pointer point to a valid `Base<T>` instance, which
	/// means it must have been properly initialized.
	///
	/// Note that a `Base<Any>` is allowed to be constructed from any valid `Base<T>` pointer, as
	/// long as you never attempt to access the contents of it (ie either through [`Gc::to_ptr`]) or
	/// through dereferencing either [`Ref`] or [`Mut`]. This is used to get header attributes
	/// for objects when the type is irrelevant.
	pub(crate) unsafe fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}

	/// Creates a new `Gc<T>` from a `Gc` referencing a base wrapping `T::Inner`.
	///
	/// As [`Base`] doesn't require an [`Allocated`] type, [`Base::new`] and friends return a
	/// `Gc<Base<T>>`. As such, the way you convert from this `Gc` to a `Gc` of the outer type is
	/// through this function.
	#[must_use]
	pub fn from_inner(inner: Gc<Base<T::Inner>>) -> Self {
		// SAFETY: This is valid, as `Allocated` guarantees that `T` and `Base<T>` are represented
		// identically, and thus converting a `Gc` of the two is valid.
		unsafe { std::mem::transmute(inner) }
	}

	/// Creates a new `Gc<t>` from the raw pointer `ptr`.
	///
	/// This is identical to [`new`], except it assumes `ptr` is nonnull. It's just for convenience.
	///
	/// # Safety
	/// All the same safety concerns as [`new`], except `ptr` may not be null.
	#[must_use]
	pub(crate) unsafe fn new_unchecked(ptr: *mut T) -> Self {
		Self::new(NonNull::new_unchecked(ptr))
	}

	/// Attempts to get an immutable reference to `self`'s contents, returning an error if it's
	/// currently mutably borrowed.
	///
	/// This function is thread-safe.
	///
	/// # Errors
	/// If the contents are already mutably borrowed (via [`Gc::as_mut`]), this will return
	/// an [`ErrorKind::AlreadyLocked`].
	///
	/// # Panics
	/// This will panic if more than [`MAX_BORROWS`] borrows are currently held.
	///
	/// # Examples
	/// Getting an immutable reference when no mutable ones exist.
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("what a wonderful day");
	///
	/// assert_eq!(*text.as_ref()?, "what a wonderful day");
	/// # quest::Result::<()>::Ok(())
	/// ```
	/// You cannot get an immutable reference when a mutable one exists.
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use quest::{error::ErrorKind, value::ty::Text};
	/// let text = Text::from_static_str("what a wonderful day");
	/// let textmut = text.as_mut()?;
	///
	/// // `textmut` is in scope, we cant get a reference.
	/// assert_matches!(text.as_ref().unwrap_err().kind(), ErrorKind::AlreadyLocked(_));
	/// drop(textmut);
	///
	/// // now it isn't, so we can get a reference.
	/// assert_eq!(*text.as_ref()?, "what a wonderful day");
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn as_ref(self) -> Result<Ref<T>> {
		if cfg!(feature="unsafe-no-locking") {
			return Ok(Ref(self))
		}

		fn updatefn(x: u32) -> Option<u32> {
			if x == MUT_BORROW {
				None
			} else {
				Some(x + 1)
			}
		}

		match self
			.borrows()
			.fetch_update(Ordering::Acquire, Ordering::Relaxed, updatefn)
		{
			Ok(x) if x == MAX_BORROWS as u32 => panic!("too many immutable borrows"),
			Ok(_) => Ok(Ref(self)),
			Err(_) => Err(crate::error::ErrorKind::AlreadyLocked(Value::from(self).any()).into()),
		}
	}

	/// Attempts to get a mutable reference to `self`'s contents, returning an error if it's
	/// currently immutably borrowed.
	///
	/// This function is thread-safe.
	///
	/// # Errors
	/// If the contents are already immutably borrowed (via [`Gc::as_ref`]), this will
	/// return an [`Error::AlreadyLocked`].
	///
	/// If the data has been [frozen](Ref::freeze), this will return a [`ErrorKind::ValueFrozen`].
	///
	/// # Examples
	/// Getting a mutable reference when no immutable ones exist.
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("what a wonderful day");
	/// let mut textmut = text.as_mut()?;
	///
	/// textmut.push('!');
	/// assert_eq!(*textmut, "what a wonderful day!");
	/// # quest::Result::<()>::Ok(())
	/// ```
	/// You cannot get a mutable reference when any immutable ones exist.
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use quest::{error::ErrorKind, value::ty::Text};
	/// let text = Text::from_static_str("what a wonderful day");
	/// let textref = text.as_ref()?;
	///
	/// // `textref` is in scope, we cant get a reference.
	/// assert_matches!(text.as_mut().unwrap_err().kind(), ErrorKind::AlreadyLocked(_));
	/// drop(textref);
	///
	/// // now it isn't, so we can get a reference.
	/// let mut textmut = text.as_mut()?;
	/// textmut.push('!');
	/// assert_eq!(*textmut, "what a wonderful day!");
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn as_mut(self) -> Result<Mut<T>> {
		if self.is_frozen() {
			return Err(crate::error::ErrorKind::ValueFrozen(Value::from(self).any()).into());
		}

		if cfg!(feature="unsafe-no-locking") {
			return Ok(Mut(self))
		}

		if self
			.borrows()
			.compare_exchange(0, MUT_BORROW, Ordering::Acquire, Ordering::Relaxed)
			.is_err()
		{
			return Err(crate::error::ErrorKind::AlreadyLocked(Value::from(self).any()).into());
		}

		let mutref = Mut(self);

		// We have to check again to see if it's frozen just in case.
		if self.is_frozen() {
			// this will drop `mutref` and thus release the mutable ownership.
			Err(crate::error::ErrorKind::ValueFrozen(Value::from(self).any()).into())
		} else {
			Ok(mutref)
		}
	}

	/// Checks to see whether `self` and `rhs` point to the same object in memory.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text1 = Text::from_static_str("Hello");
	/// let text2 = Text::from_static_str("Hello");
	/// let text3 = text1;
	///
	/// assert!(text1.ptr_eq(text3));
	/// assert!(!text1.ptr_eq(text2));
	/// ```
	#[must_use]
	pub fn ptr_eq(self, rhs: Self) -> bool {
		self.0 == rhs.0
	}

	/// Checks to see whether the object is currently frozen.
	///
	/// Frozen objects are unable to be [mutably accessed](Gc::as_mut), and are frozen via
	/// [`Ref::freeze`].
	///
	/// # Examples
	/// ```
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use quest::{error::ErrorKind, value::ty::Text};
	/// let text = Text::from_static_str("Quest is cool");
	///
	/// text.as_ref()?.freeze();
	/// assert!(text.is_frozen());
	/// assert_matches!(text.as_mut().unwrap_err().kind(), ErrorKind::ValueFrozen(_));
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[must_use]
	pub fn is_frozen(&self) -> bool {
		self.flags().contains(Flags::FROZEN)
	}

	/// Converts `self` into a pointer to the base.
	#[must_use]
	pub(crate) fn as_ptr(self) -> *const T {
		self.0.as_ptr()
	}

	/// Gets the flags of `self`.
	///
	/// Technically this could be publicly visible, but outside the crate, you should get a reference
	/// and go through the [`Header`].
	fn flags(&self) -> &Flags {
		unsafe { &*self.as_ptr() }.header().flags()
	}

	/// Gets the header of `self`.
	///
	/// Technically this could be publicly visible, but outside the crate, you should get a reference
	/// and go through the [`Header`].
	fn borrows(&self) -> &AtomicU32 {
		// SAFETY: we know `self.as_ptr()` always points to a valid `Base<T>`, as that's a requirement
		// for constructing it (via `new`).
		unsafe { &*self.as_ptr() }.header().borrows()
	}

	pub fn call_attr<A: Attribute>(&self, attr: A, args: crate::vm::Args<'_>) -> Result<AnyValue> {
		// try to get a function directly defined on `self`, which most likely wont exist.
		// then, if it doesnt, call the `parents.call_attr`, which is more specialized.
		let obj = self.to_any();
		let asref = self.as_ref()?;

		if let Some(func) = asref.attributes().get_unbound_attr(attr)? {
			drop(asref);
			return func.call(args.with_self(obj));
		}

		let attr = asref
			.parents()
			.get_unbound_attr_checked(attr, &mut Vec::new())?
			.ok_or_else(|| crate::error::ErrorKind::UnknownAttribute(obj, attr.to_value()))?;

		drop(asref);
		attr.call(args.with_self(obj))
	}
}

impl<T: Allocated> From<Gc<T>> for Value<Gc<T>> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		sa::const_assert_eq!(std::mem::align_of::<Base<Any>>(), 16);

		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b1111, 0, "somehow the `Base<T>` pointer was misaligned??");

		// SAFETY: The bottom three bits being zero is the definition for `Gc<T>`. We know that the
		// bottom three bits are zero because `Base<T>` will always be at least 8-aligned.
		unsafe { Self::from_bits_unchecked(bits) }
	}
}

// SAFETY: We correctly implemented `is_a` to only return true if the `AnyValue` is a `Gc<T>`.
// Additionally, `get` will always return a valid `Gc<T>` for any `Value<Gc<T>>`.
unsafe impl<T: Allocated> Convertible for Gc<T> {
	#[inline]
	fn is_a(value: AnyValue) -> bool {
		// If the `value` isn't allocated, it's not a `Gc`.
		if !value.is_allocated() {
			return false;
		}

		// SAFETY: Since `value` is allocated, we know it could only have come from a valid `Gc`. As
		// such, converting the bits to a pointer will yield a non-zero pointer. Additionally, since
		// the pointer points to _some_ `Gc` type, we're allowed to construct a `Gc<Any>` of it, as
		// we're not accessing the `data` at all. (We're only getting the `typeid` from the header.)
		let typeid = unsafe {
			let gc = Gc::new_unchecked(value.bits() as usize as *mut Wrap<Any>);
			Base::_typeid(gc.as_ptr().cast())
		};

		// Make sure the `typeid` matches that of `T`.
		typeid == std::any::TypeId::of::<T::Inner>()
	}

	fn get(value: Value<Self>) -> Self {
		// SAFETY: The only way to get a `Value<Gc<T>>` is either through `Value::from` (which by
		// definition constructs a valid `Value` from a valid `Gc<T>` or through `Gc::downcast`, which
		// will only return `Some` if the underlying value is a `Gc<T>` (via `Gc::is_a`). Thus, we
		// know that the bits are guaranteed to be a valid pointer to a `Base<T>`.
		unsafe { Self::new_unchecked(value.bits() as usize as *mut T) }
	}
}

/// A smart pointer used to release read access when dropped.
///
/// This is created via the [`as_ref`](Gc::as_ref) method on [`Gc`].
#[repr(transparent)]
pub struct Ref<T: Allocated>(Gc<T>);

impl<T: Allocated + Debug> Debug for Ref<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&**self, f)
	}
}

impl<T: Allocated> Ref<T> {
	#[must_use]
	pub fn as_gc(&self) -> Gc<T> {
		self.0
	}

	pub fn call_attr<A: Attribute>(&self, attr: A, args: crate::vm::Args<'_>) -> Result<AnyValue> {
		// try to get a function directly defined on `self`, which most likely wont exist.
		// then, if it doesnt, call the `parents.call_attr`, which is more specialized.
		let obj = self.as_gc().to_any();

		if let Some(func) = self.attributes().get_unbound_attr(attr)? {
			func.call(args.with_self(obj))
		} else {
			self.parents().call_attr(obj, attr, args)
		}
	}

	pub fn get_unbound_attr_checked<A: Attribute>(&self, attr: A, checked: &mut Vec<AnyValue>) -> Result<Option<AnyValue>> {
		self.header().get_unbound_attr_checked(attr, checked)
	}

	#[must_use]
	pub fn flags(&self) -> &Flags {
		self.header().flags()
	}

	pub fn freeze(&self) {
		self.header().freeze();
	}

	pub fn parents(&self) -> ParentsRef<'_> {
		self.header().parents()
	}

	pub fn attributes(&self) -> AttributesRef<'_> {
		self.header().attributes()
	}
}

impl<T: Allocated> Clone for Ref<T> {
	fn clone(&self) -> Self {
		let gcref_result = self.as_gc().as_ref();

		// SAFETY: We currently have an immutable reference to a `Ref`, so
		// we know that no mutable ones can exist.
		unsafe { gcref_result.unwrap_unchecked() }
	}
}

impl<T: Allocated> Deref for Ref<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		// SAFETY: When a `Gc` is constructed, it must have been passed an initialized `Base<T>`,
		// which means that its `data` must also have been initialized.
		unsafe { &*self.0.as_ptr() }
	}
}

impl<T: Allocated> Drop for Ref<T> {
	fn drop(&mut self) {
		let prev = self.0.borrows().fetch_sub(1, Ordering::Release);

		// Sanity check, as it's impossible for us to have a `MUT_BORROW` after a `Ref` is created.
		debug_assert_ne!(prev, MUT_BORROW);

		// Another sanity check, as this indicates something double freed (or a `Mut` was
		// incorrectly created).
		debug_assert_ne!(prev, 0);
	}
}

/// A smart pointer used to release write access when dropped.
///
/// This is created via the [`as_mut`](Gc::as_mut) method on [`Gc`].
#[repr(transparent)]
pub struct Mut<T: Allocated>(Gc<T>);

impl<T: Debug + Allocated> Debug for Mut<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&**self, f)
	}
}

impl<T: Allocated> Mut<T> {
	pub fn parents_mut(&mut self) -> ParentsMut<'_> {
		self.header_mut().parents_mut()
	}

	pub fn attributes_mut(&mut self) -> AttributesMut<'_> {
		self.header_mut().attributes_mut()
	}

	pub fn parents_list(&mut self) -> Gc<List> {
		self.parents_mut().as_list()
	}

	pub fn set_parents<P: IntoParent>(&mut self, parents: P) {
		self.parents_mut().set(parents);
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		self.header_mut().set_attr(attr, value)
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		self.header_mut().del_attr(attr)
	}
}

/*impl<T: Allocated> Mut<T> {
	/// Converts a [`Mut`] to a [`Ref`].
	///
	/// Just as you're able to downgrade mutable references to immutable ones in Rust (eg you can do
	/// `(&mut myvec).len()`), you're able to downgrade mutable [`Gc`] references to immutable ones.
	/// However, since Mut implements both [`Deref<Target=T>`] and [`DerefMut<Target=T>`], Rust
	/// won't let us _also_ have [`Deref<Target=Ref<T>>`]; this method exists to provide that
	/// functionality. (The short name is intended to make it as painless as possible to cast to a
	/// [`Ref<T>`].)
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("Quest is cool");
	/// let mut textmut = text.as_mut()?;
	/// textmut.push('!');
	///
	/// // Text only defines `len` on `Ref<Text>`. Thus, we
	/// // need to convert reference before we can call `len`.
	/// assert_eq!(textmut.r().len(), 14);
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[inline]
	pub fn r(&self) -> &Ref<T> {
		// SAFETY: both `Mut` and `Ref` have the same internal layout. Additionally, since we
		// return a reference to the `Ref`, its `Drop` won't be called.
		unsafe { &*(self as *const Self).cast() }
	}
}*/

impl<T: Allocated> Deref for Mut<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.0.as_ptr() }
	}
}

impl<T: Allocated> DerefMut for Mut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// SAFETY: When a `Gc` is constructed, it must have been passed an initialized `Base<T>`,
		// which means that its `data` must also have been initialized. Additionally, we have unique
		// access over `data`, so we can mutably borrow it
		unsafe { &mut *(self.0.as_ptr() as *mut Self::Target) }
	}
}

impl<T: Allocated> Drop for Mut<T> {
	fn drop(&mut self) {
		if cfg!(debug_assertions) {
			// Sanity check to ensure that the value was previously `MUT_BORROW`
			debug_assert_eq!(MUT_BORROW, self.0.borrows().swap(0, Ordering::Release));
		} else {
			self.0.borrows().store(0, Ordering::Release);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::Text;

	#[should_panic = "too many immutable borrows"]
	#[test]
	fn too_many_immutable_borrows_cause_a_panick() {
		let text = Text::from_static_str("g'day mate");

		text.borrows().store(MAX_BORROWS as u32, Ordering::Release);

		let _ = text.as_ref();
	}

	#[test]
	fn respects_refcell_rules() {
		let text = Text::from_static_str("g'day mate");

		let mut1 = text.as_mut().unwrap();
		assert_matches!(text.as_ref().unwrap_err().kind(), crate::error::ErrorKind::AlreadyLocked(_));
		drop(mut1);

		let ref1 = text.as_ref().unwrap();
		assert_matches!(text.as_mut().unwrap_err().kind(), crate::error::ErrorKind::AlreadyLocked(_));

		let ref2 = text.as_ref().unwrap();
		assert_matches!(text.as_mut().unwrap_err().kind(), crate::error::ErrorKind::AlreadyLocked(_));

		drop(ref1);
		assert_matches!(text.as_mut().unwrap_err().kind(), crate::error::ErrorKind::AlreadyLocked(_));

		drop(ref2);
		assert_matches!(text.as_mut(), Ok(_));
	}

	#[test]
	fn respects_frozen() {
		let text = Text::from_static_str("Hello, world");

		text.as_mut().unwrap().push('!');
		assert_eq!(*text.as_ref().unwrap(), *"Hello, world!");
		assert!(!text.is_frozen());

		text.as_ref().unwrap().freeze();
		assert_matches!(text.as_mut().unwrap_err().kind(), crate::error::ErrorKind::ValueFrozen(_));
		assert!(text.is_frozen());
	}
}
