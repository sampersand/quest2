//! Types related to allocated Quest types.

use crate::{Result, Error};
use crate::value::base::{Base, Header, Flags};
use crate::value::{AnyValue, Convertible, Value, value::Any};
use crate::value::ty::Wrap;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) type AnyGc = Gc<Wrap<Any>>;

/// A garbage collected pointer to `T`.
///
/// All non-immediate types in Quest are allocated on the heap. These types can never be accessed
/// directly, but must be interacted with through [`Gc`] or its references ([`GcRef`] and [`GcMut`]).
///
/// Heap-allocated types will define all their methods either on [`Gc`] directly (for associated
/// functions, eg [`Gc<Text>::from_str`]), or [`GcRef`]/[`GcMut`]
/// depending on whether they need immutable (eg [`GcRef<Text>::as_str`]) or mutable (eg
/// [`GcMut<Text>::push_str`]) access.
/// 
/// # Examples
/// ```rust
/// # use qvm_rt::value::{gc::{Gc, GcRef, GcMut}, ty::Text};
/// # fn main() -> qvm_rt::Result<()> {
/// let text = Text::from_str("Quest is cool");
/// 
/// let textref: GcRef<Text> = text.as_ref()?;
/// assert_eq!(textref.as_str(), "Quest is cool");
///
/// drop(textref);
/// let mut textmut: GcMut<Text> = text.as_mut()?;
/// textmut.push('!');
/// 
/// assert_eq!(textmut.as_str(), "Quest is cool!");
/// # Ok(()) }
/// ```
#[repr(transparent)]
pub struct Gc<T: Allocated>(NonNull<T>);

unsafe impl<T: Allocated + Send> Send for Gc<T> {}
unsafe impl<T: Allocated + Sync> Sync for Gc<T> {}

/// A trait that indicates a type contains at a minimum a [`Header`].
///
/// # Safety
/// To correctly implement this trait, the struct must guarantee that you can safely cast a 
/// pointer to `Self` to a pointer to [`Header`]—that is, the first struct is `#[repr(C)]`, and
/// the first field is a [`Header`]. This can be trivially achieved if your struct is simply a
/// `#[repr(transparent)]` wrapper around [`Base<...>`](Base).
pub unsafe trait Allocated: 'static {
	#[doc(hidden)]
	fn _inner_typeid() -> std::any::TypeId;

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
	GcRef<T>: Debug,
{
	fn fmt(self: &Gc<T>, f: &mut Formatter) -> fmt::Result {
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

/*
impl<T: HasParents + Allocated> Gc<T> {
	/// Helper function for `Base::allocate`. See it for safety.
	pub(crate) unsafe fn allocate() -> Builder<T> {
		Base::allocate()
	}
*/

/// Sentinel value used to indicate the `Gc<T>` is mutably borrowed.
const MUT_BORROW: u32 = u32::MAX;

impl<T: Allocated> Gc<T> {
	/// Creates a new `Gc<T>` from `ptr`.
	///
	/// # Safety
	/// The `Base<T>` must have been allocated via `crate::alloc`. Additionally, the pointer 
	/// point to a valid `Base<T>` instance, which means it must have been properly initialized.
	/// 
	/// Note that a `Base<Any>` is allowed to be constructed from any valid `Base<T>` pointer, as
	/// long as you never attempt to access the contents of it (ie either through [`Gc::to_ptr`]) or
	/// through dereferencing either [`GcRef`] or [`GcMut`]. This is used to get header attributes
	/// for objects when the type is irrelevant.
	pub(crate) unsafe fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}

	/// Creates a new `Gc<t>` from the raw pointer `ptr`.
	/// 
	/// This is identical to [`new`], except it assumes `ptr` is nonnull. It's just for convenience.
	///
	/// # Safety
	/// All the same safety concerns as [`new`], except `ptr` may not be null.
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
	/// an [`Error::AlreadyLocked`].
	///
	/// # Examples
	/// Getting an immutable reference when no mutable ones exist.
	/// ```rust
	/// # use qvm_rt::value::ty::Text;
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("what a wonderful day");
	/// 
	/// assert_eq!(text.as_ref()?.as_str(), "what a wonderful day");
	/// # Ok(()) }
	/// ```
	/// You cannot get an immutable reference when a mutable one exists.
	/// ```rust
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("what a wonderful day");
	/// let textmut = text.as_mut()?;
	/// 
	/// // `textmut` is in scope, we cant get a reference.
	/// assert_matches!(text.as_ref(), Err(Error::AlreadyLocked(_)));
	/// drop(textmut);
	/// 
	/// // now it isn't, so we can get a reference.
	/// assert_eq!(text.as_ref()?.as_str(), "what a wonderful day");
	/// # Ok(()) }
	/// ```
	pub fn as_ref(self) -> Result<GcRef<T>> {
		fn updatefn(x: u32) -> Option<u32> {
			if x == MUT_BORROW {
				None
			} else {
				Some(x + 1)
			}
		}

		const ONE_BELOW_MUT_BORROW: u32 = MUT_BORROW - 1;

		match self.borrows().fetch_update(Ordering::Acquire, Ordering::Relaxed, updatefn) {
			Ok(ONE_BELOW_MUT_BORROW) => panic!("too many immutable borrows"),
			Ok(_) => Ok(GcRef(self)),
			Err(_) => Err(Error::AlreadyLocked(Value::from(self).any())),
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
	/// If the data has been [frozen](GcRef::freeze), this will return a [`Error::ValueFrozen`].
	///
	/// # Examples
	/// Getting a mutable reference when no immutable ones exist.
	/// ```rust
	/// # use qvm_rt::value::ty::Text;
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("what a wonderful day");
	/// let mut textmut = text.as_mut()?;
	///
	/// textmut.push('!');
	/// assert_eq!(textmut.as_str(), "what a wonderful day!");
	/// # Ok(()) }
	/// ```
	/// You cannot get a mutable reference when any immutable ones exist.
	/// ```rust
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("what a wonderful day");
	/// let textref = text.as_ref()?;
	/// 
	/// // `textref` is in scope, we cant get a reference.
	/// assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));
	/// drop(textref);
	/// 
	/// // now it isn't, so we can get a reference.
	/// let mut textmut = text.as_mut()?;
	/// textmut.push('!');
	/// assert_eq!(textmut.as_str(), "what a wonderful day!");
	/// Ok(()) }
	/// ```
	pub fn as_mut(self) -> Result<GcMut<T>> {
		if self.flags().contains(Flags::FROZEN) {
			return Err(Error::ValueFrozen(Value::from(self).any()))
		}

		if self
			.borrows()
			.compare_exchange(0, MUT_BORROW, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
		{
			Ok(GcMut(self))
		} else {
			Err(Error::AlreadyLocked(Value::from(self).any()))
		}
	}

	/// Checks to see whether `self` and `rhs` point to the same object in memory.
	///
	/// # Examples
	/// ```rust
	/// # use qvm_rt::value::ty::Text;
	/// let text1 = Text::from_str("Hello");
	/// let text2 = Text::from_str("Hello");
	/// let text3 = text1;
	///
	/// assert!(text1.ptr_eq(text3));
	/// assert!(!text1.ptr_eq(text2));
	/// ```
	pub fn ptr_eq(self, rhs: Self) -> bool {
		self.0 == rhs.0
	}

	/// Checks to see whether the object is currently frozen.
	///
	/// Frozen objects are unable to be [mutably accessed](Gc::as_mut), and are frozen via
	/// [`GcRef::freeze`].
	///
	/// # Examples
	/// ```rust
	/// # #[macro_use] use assert_matches::assert_matches;
	/// # use qvm_rt::{Error, value::ty::Text};
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Text::from_str("Quest is cool");
	/// 
	/// text.as_ref()?.freeze();
	/// assert!(text.is_frozen());
	/// assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
	/// # Ok(()) }
	/// ```
	pub fn is_frozen(&self) -> bool {
		self.flags().contains(Flags::FROZEN)
	}

	/// Converts `self` into a pointer to the base.
	pub(crate) fn as_ptr(self) -> *const T {
		self.0.as_ptr()
	}

	/// Gets the flags of `self`.
	///
	/// Technically this could be publicly visible, but outside the crate, you should get a reference
	/// and go through the [`Header`].
	fn flags(&self) -> &Flags {
		unsafe { &*self.0.as_ptr() }.header().flags()
	}

	/// Gets the header of `self`.
	///
	/// Technically this could be publicly visible, but outside the crate, you should get a reference
	/// and go through the [`Header`].
	fn borrows(&self) -> &AtomicU32 {
		// SAFETY: we know `self.as_ptr()` always points to a valid `Base<T>`, as that's a requirement
		// for constructing it (via `new`).
		unsafe { &*self.0.as_ptr() }.header().borrows()
	}
}

impl<T: Allocated> From<Gc<T>> for Value<Gc<T>> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		sa::assert_eq_align!(Base<Any>, u64);

		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b111, 0, "somehow the `Base<T>` pointer was misaligned??");

		// SAFETY: The bottom three bits being zero is the definition for `Gc<T>`. We know that the
		// bottom three bits are zero because `Base<T>` will always be at least 8-aligned.
		unsafe { Self::from_bits_unchecked(bits) }
	}
}

// SAFETY: We correctly implemented `is_a` to only return true if the `AnyValue` is a `Gc<T>`.
// Additionally, `get` will always return a valid `Gc<T>` for any `Value<Gc<T>>`.
unsafe impl<T: Allocated> Convertible for Gc<T>
where
	GcRef<T>: Debug,
{
	type Output = Self;

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
			*std::ptr::addr_of!((*(gc.as_ptr() as *const Base<Any>)).header.typeid)
		};

		dbg!(value.bits() as usize as *mut Wrap<Any>);
		// dbg!(typeid, std::any::TypeId::of::<T>(), std::any::TypeId::of::<crate::value::ty::Text>());

		// Make sure the `typeid` matches that of `T`.
		typeid == T::_inner_typeid()
	}

	fn get(value: Value<Self>) -> Self {
		// SAFETY: The only way to get a `Value<Gc<T>>` is either through `Value::from` (which by
		// definition constructs a valid `Value` from a valid `Gc<T>` or through `Gc::downcast`, which
		// will only return `Some` if the underlying value is a `Gc<T>` (via `Gc::is_a`). Thus, we
		// know that the bits are guaranteed to be a valid pointer to a `Base<T>`.
		unsafe { Gc::new_unchecked(value.bits() as usize as *mut T) }
	}
}

/// A smart pointer used to release read access when dropped.
///
/// This is created via the [`as_ref`](Gc::as_ref) method on [`Gc`].
#[repr(transparent)]
pub struct GcRef<T: Allocated>(Gc<T>);

impl<T: Allocated + Debug> Debug for GcRef<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.deref(), f)
	}
}

impl<T: Allocated> GcRef<T> {
	pub fn as_gc(&self) -> Gc<T> {
		self.0
	}

	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.header().get_attr(attr)
	}

	pub fn flags(&self) -> &Flags {
		self.header().flags()
	}

	pub fn freeze(&self) {
		self.header().freeze()
	}
}

impl<T: Allocated> Clone for GcRef<T> {
	fn clone(&self) -> Self {
		let gcref_result = self.as_gc().as_ref();

		// SAFETY: We currently have an immutable reference to a `GcRef`, so
		// we know that no mutable ones can exist.
		unsafe { gcref_result.unwrap_unchecked() }
	}
}

impl<T: Allocated> Deref for GcRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		// SAFETY: When a `Gc` is constructed, it must have been passed an initialized `Base<T>`,
		// which means that its `data` must also have been initialized.
		unsafe { &*(self.0).0.as_ptr() }
	}
}

impl<T: Allocated> Drop for GcRef<T> {
	fn drop(&mut self) {
		let prev = self.0.borrows().fetch_sub(1, Ordering::Release);

		// Sanity check, as it's impossible for us to have a `MUT_BORROW` after a `GcRef` is created.
		debug_assert_ne!(prev, MUT_BORROW);

		// Another sanity check, as this indicates something double freed (or a `GcMut` was
		// incorrectly created).
		debug_assert_ne!(prev, 0);
	}
}

/// A smart pointer used to release write access when dropped.
///
/// This is created via the [`as_mut`](Gc::as_mut) method on [`Gc`].
#[repr(transparent)]
pub struct GcMut<T: Allocated>(Gc<T>);

impl<T: Debug + Allocated> Debug for GcMut<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.deref(), f)
	}
}


impl<T: Allocated> GcMut<T> {
	pub fn parents(&mut self) -> crate::value::gc::Gc<crate::value::ty::List> {
		self.header_mut().parents()
	}

	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		self.header_mut().set_attr(attr, value)
	}

	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.header_mut().del_attr(attr)
	}
}

/*
impl<T: Allocated> GcMut<T> {
	fn _base_mut(&self) -> &mut Base<T> {
		// SAFETY: When a `Gc` is constructed, it must have been passed an initialized `Base<T>`.
		// Additionally, since we have a unique lock on the data, we can get a mutable pointer.
		unsafe { &mut *(self.0).0.as_ptr() }
	}

	/// Converts a [`GcMut`] to a [`GcRef`].
	///
	/// Just as you're able to downgrade mutable references to immutable ones in Rust (eg you can do
	/// `(&mut myvec).len()`), you're able to downgrade mutable [`Gc`] references to immutable ones.
	/// However, since GcMut implements both [`Deref<Target=T>`] and [`DerefMut<Target=T>`], Rust
	/// won't let us _also_ have [`Deref<Target=GcRef<T>>`]; this method exists to provide that
	/// functionality. (The short name is intended to make it as painless as possible to cast to a
	/// [`GcRef<T>`].)
	///
	/// # Examples
	/// # use qvm_rt::{Error, value::Gc};
	/// # fn main() -> qvm_rt::Result<()> {
	/// let text = Gc::from_str("Quest is cool");
	/// let mut textmut = text.as_mut()?;
	/// textmut.push('!');
	/// 
	/// // Text only defines `as_str` on `GcRef<Text>`. Thus, we
	/// // need to convert reference before we can call `as_str`.
	/// assert_eq!(text.as_mut(), "Quest is cool!");
	/// # Ok(()) }
	#[inline(always)]
	pub fn r(&self) -> &GcRef<T> {
		// SAFETY: both `GcMut` and `GcRef` have the same internal layout. Additionally, since we
		// return a reference to the `GcRef`, its `Drop` won't be called.
		unsafe { std::mem::transmute(self) }
	}

	// Gets a mutable reference to this `self`'s header.
	fn _header_mut(&mut self) -> &mut crate::value::base::Header {
		&mut self._base_mut().header
	}
}*/

impl<T: Allocated> Deref for GcMut<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*(self.0).0.as_ptr() }
	}
}

impl<T: Allocated> DerefMut for GcMut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// SAFETY: When a `Gc` is constructed, it must have been passed an initialized `Base<T>`,
		// which means that its `data` must also have been initialized. Additionally, we have unique
		// access over `data`, so we can mutably borrow it
		unsafe { &mut *(self.0).0.as_ptr() }
	}
}

impl<T: Allocated> Drop for GcMut<T> {
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

	#[should_panic="too many immutable borrows"]
	#[test]
	fn too_many_immutable_borrows_cause_a_panick() {
		let text = Text::from_str("g'day mate");

		text.borrows().store(MUT_BORROW - 1, Ordering::Release);

		let _ = text.as_ref();
	}

	#[test]
	fn respects_refcell_rules() {
		let text = Text::from_str("g'day mate");

		let mut1 = text.as_mut().unwrap();
		assert_matches!(text.as_ref(), Err(Error::AlreadyLocked(_)));
		drop(mut1);

		let ref1 = text.as_ref().unwrap();
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		let ref2 = text.as_ref().unwrap();
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		drop(ref1);
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		drop(ref2);
		assert_matches!(text.as_mut(), Ok(_));
	}

	#[test]
	fn respects_frozen() {
		let text = Text::from_str("Hello, world");

		text.as_mut().unwrap().push('!');
		assert_eq!(*text.as_ref().unwrap(), *"Hello, world!");
		assert!(!text.is_frozen());

		text.as_ref().unwrap().freeze();
		assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
		assert!(text.is_frozen());
	}
}
