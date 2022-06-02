//! The string representation within quest.

use crate::value::base::Flags;
use crate::value::gc::{Allocated, Gc};
#[allow(unused)]
use crate::value::ty::List;
use crate::value::Intern;
use crate::vm::Args;
use crate::{Result, ToValue, Value};
use std::alloc;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

mod builder;
mod simple_builder;

pub use builder::Builder;
pub use simple_builder::SimpleBuilder;

quest_type! {
	/// The type that represents text (ie Strings) in Quest.
	///
	/// Note that `Text`s must contain valid utf-8 (just like [`str`])s. This means that a [`List`]
	/// must be used for arbitrary series of bytes. See the [`str`] docs for more details on this.
	///
	/// For efficiency's sake, there's multiple ways to create strings, such as from
	/// [a `&'static str`](Text::from_static_str), [a `String`](Text::from_string), or, for the
	/// finest control, you can use [`Builder`] ([`Text::builder`] is a provided shorthand).
	///
	/// Because attribute keys are commonly `Text`s, `Text`s store their [`hash`](Text::fast_hash)
	/// internally for faster lookup and comparisons.
	///
	/// # `Gc<Text>`
	/// Note that you can never construct a `Text` by-value; it must always be wrapped in a [`Gc`].
	/// This is to make it compatible with other [`Allocated`](crate::value::Base::Allocated). As
	/// such, you'll need to go through [`Gc::as_ref`] or [`Gc::as_mut`] if you want to access the
	/// methods defined on `Text`.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("Hello");
	///
	/// // Immutably borrow it. Since we just created it,
	/// // nothing should have a mutable reference.
	/// assert_eq!(*text.as_ref().unwrap(), "Hello");
	///
	/// // Mutably borrow it and then add something to it.
	/// let mut textmut = text.as_mut().unwrap();
	/// textmut.push_str(", world");
	/// textmut.push('!');
	/// assert_eq!(*textmut, "Hello, world!");
	///
	/// // We're unable to mutably borrow it as `textmut` is
	/// // still in scope.
	/// assert!(text.as_ref().is_err());
	///
	/// // Dropping `textmut` allows us to reference it again.
	/// drop(textmut);
	/// assert_eq!(*text.as_ref().unwrap(), "Hello, world!");
	/// ```
	#[derive(NamedType)]
	pub struct Text(Inner);
}

impl super::AttrConversionDefined for Gc<Text> {
	const ATTR_NAME: Intern = Intern::at_text;
}

// #[macro_export]
// macro_rules! static_text {
// 	($text:expr) => {
// 		$crate::value::ty::Text($crate::value::base::Base {
// 			header: $crate::value::base::Header {
// 				typeid: ::std::to_value::TypeId::of::<$crate::value::ty::text::Inner>(),
// 				parents: $crate::value::base::Parents::NONE,
// 				attributes: $crate::value::base::Attributes::NONE,
// 				flags: $crate::value::base::Flags::new(0),
// 				borrows: ::std::sync::atomic::AtomicU32::new(0)
// 			},
// 			data: ::std::cell::UnsafeCell::new(::std::mem::MaybeUninit::new(Inner {
// 				embed: $crate::value::ty::text::EmbeddedText {
// 					buf: [b'0'; MAX_EMBEDDED_LEN]
// 				}
// 			}))
// 		})
// 	};
// }

// static EMPTY: Text = static_text!(b"");
// static EMPTY: UnsafeCell<MaybeUninit<Text>> = UnsafeCell::new(MaybeUninit::zeroed());

#[repr(C)]
#[doc(hidden)]
pub union Inner {
	alloc: AllocatedText,
	embed: EmbeddedText,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

#[repr(C)]
#[derive(Clone, Copy)]
struct AllocatedText {
	hash: u64,
	len: usize,
	cap: usize,
	ptr: *mut u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedText {
	hash: u64,
	buf: [u8; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedText>() - std::mem::size_of::<u64>();
const FLAG_EMBEDDED: u32 = Flags::USER0;
const FLAG_SHARED: u32 = Flags::USER1;
const FLAG_NOFREE: u32 = Flags::USER2;
const FLAG_FROM_STRING: u32 = Flags::USER3;
const EMBED_LENMASK: u32 = Flags::USER1 | Flags::USER2 | Flags::USER3 | Flags::USER4 | Flags::USER5;

const _: () = assert!(MAX_EMBEDDED_LEN <= unmask_len(EMBED_LENMASK));

const fn unmask_len(len: u32) -> usize {
	debug_assert!(len & !EMBED_LENMASK == 0);
	(len >> 1) as usize
}

const fn mask_len(len: usize) -> u32 {
	debug_assert!(len <= MAX_EMBEDDED_LEN);
	(len as u32) << 1
}

fn alloc_ptr_layout(cap: usize) -> alloc::Layout {
	alloc::Layout::array::<u8>(cap).unwrap()
}

/// A hash function that prioritizes speed over uniqueness.
///
/// Because [`Text`]s are frequently used as attribute keys, repeated hashes and comparisons on them
/// is common. As such, the [hash of a `Text`](Text::fast_hash) is cached for faster lookup.
///
/// Currently murmur hash is used, but do not rely on this.
///
/// # See Also
/// - [`fast_hash_continue`] for continuing the hash of an earlier `str`.
///
/// # Examples
/// ```
/// use quest::value::ty::{Text, text::fast_hash};
///
/// let greeting = "hello, world";
/// let hash = fast_hash(greeting);
/// let text = Text::from_str(greeting);
///
/// assert_eq!(hash, text.as_ref().unwrap().fast_hash());
/// ```
#[must_use]
pub const fn fast_hash(input: &str) -> u64 {
	fast_hash_continue(FAST_HASH_START, input)
}

/// The initial value that should be passed to [`fast_hash_continue`].
///
/// This magic number comes from the murmur hash design itself.
pub const FAST_HASH_START: u64 = 525201411107845655;

/// Continue hashing where you left off.
///
/// Sometimes you need to hash an input that isn't necessarily contiguous (for example,
/// checking the hash that two concatenated [`Text`]s _would_ have). This function let's you
/// hash in a piecewise fashion.
///
/// Note that the initial `hash` must begin with [`FAST_HASH_START`].
///
/// If you don't need to hash in parts, use [`fast_hash`] instead.
///
/// # Examples
/// ```
/// use quest::value::ty::text::{
///    fast_hash, fast_hash_continue, FAST_HASH_START
/// };
///
/// let mut hash = FAST_HASH_START;
/// hash = fast_hash_continue(hash, "hello");
/// hash = fast_hash_continue(hash, ", ");
/// hash = fast_hash_continue(hash, "world");
///
/// assert_eq!(hash, fast_hash("hello, world"));
/// ```
#[must_use]
pub const fn fast_hash_continue(mut hash: u64, input: &str) -> u64 {
	let bytes = input.as_bytes();
	let mut idx = 0;

	// `for` in const fns is not stable, so we use `while`.
	while idx < bytes.len() {
		hash ^= bytes[idx] as u64;
		hash = hash.wrapping_mul(0x5bd1e9955bd1e995);
		hash ^= hash >> 47;

		idx += 1;
	}

	hash
}

impl Text {
	// Helper function for fetching `Inner`.
	fn inner(&self) -> &Inner {
		self.0.data()
	}

	// Helper function for fetching `Inner` mutably.
	fn inner_mut(&mut self) -> &mut Inner {
		self.0.data_mut()
	}

	/// A helper function that simply returns [`Builder::allocate`].
	///
	/// # Examples
	/// ```
	/// use quest::value::ty::Text;
	///
	/// let mut builder = Text::builder();
	/// // ... use the builder
	/// # let _ = builder;
	/// ```
	pub fn builder() -> Builder {
		Builder::allocate()
	}

	/// A helper function that simply returns [`SimpleBuilder::new`].
	///
	/// # See Also
	/// - [`Builder`] for more fine-grained control over building a [`Text`].
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let mut builder = Text::simple_builder();
	///
	/// builder.push_str("Hello");
	/// builder.push_str(", world");
	/// builder.push('!');
	/// let text = builder.finish();
	///
	/// assert_eq!(*text.as_ref().unwrap(), "Hello, world!");
	/// ```
	pub fn simple_builder() -> SimpleBuilder {
		SimpleBuilder::new()
	}

	/// Creates a new, empty [`Text`].
	///
	/// If you have an idea of the required capacity, consider calling [`Text::with_capacity`] or
	/// [`Text::SimpleBuilder`] instead.. For finer-tuned construction, see [`Text::builder`].
	///
	/// Note that this will still allocate memory for the underlying [`Text`] object, but it won't
	/// allocate a separate buffer.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::new();
	/// assert!(text.as_ref()?.is_empty());
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[must_use]
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	/// Creates a new, empty [`Text`] with at least the given capacity.
	///
	/// To use this you have to call [`Gc::as_mut`]; to skip this step and just initialize directly,
	/// you can use [`Text::simple_builder`].
	///
	/// Note that this will still allocate memory for the underlying [`Text`] object regardless of
	/// the capacity.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::with_capacity(13);
	///
	/// let mut textmut = text.as_mut()?;
	/// textmut.push_str("Hello, ");
	/// textmut.push_str("world");
	/// textmut.push('!');
	///
	/// assert_eq!(*textmut, "Hello, world!");
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[must_use]
	pub fn with_capacity(capacity: usize) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.allocate_buffer(capacity);
		builder.finish() // Nothing else to do, as the default state is valid.
	}

	/// Creates a new [`Text`] from the given `&str`.
	///
	/// Note that because we do not own the `text`, a new buffer may potentially be allocated. If
	/// you have a [`String`], use [`Text::from_string`] so as to reuse the buffer, and for
	/// `&'static str`s, use [`Text::from_static_str`] which will reference the buffer.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let fruit = "Banana";
	///
	/// // You really should use `from_static_str` as `fruit`
	/// // is `'static`, but this is just an example.
	/// let text = Text::from_str(fruit);
	///
	/// assert_eq!(*text.as_ref()?, "Banana");
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[allow(clippy::should_implement_trait)]
	#[must_use]
	pub fn from_str(text: &str) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.allocate_buffer(text.len());

		// SAFETY: We just allocated enough storage, so we know this can fit
		// additionally, `builder.finish()` will set the hash.
		unsafe {
			builder.text_mut().push_str_unchecked(text);
		}

		builder.finish()
	}

	/// Creates a new [`Text`] from the given `&'static str`.
	///
	/// Note that because `text` lives for the lifetime of the program and is guaranteed not to
	/// change, this function will simply reference it and not allocate a new buffer. As such, this
	/// is preferred to [`Text::from_str`] which may potentially allocate a new buffer.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let fruit = "Orange";
	/// let text = Text::from_static_str(fruit);
	/// assert_eq!(*text.as_ref()?, "Orange");
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[must_use]
	pub fn from_static_str(text: &'static str) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flags(FLAG_NOFREE | FLAG_SHARED);

		// SAFETY: Both embed and alloc are valid when zero initialized.
		let mut alloc = unsafe { &mut builder.inner_mut().alloc };

		// Even though we cast it to a `*mut u8`, the `FLAG_SHARED` ensures we won't be modifying it.
		alloc.ptr = text.as_ptr() as *mut u8;
		alloc.len = text.len();
		alloc.cap = alloc.len; // The capacity is the same as the length.

		builder.finish()
	}

	/// Creates a new [`Text`] from the given [`String`].
	///
	/// Note that because [`String`] owns its buffer, we're able to salvage that and not allocate an
	/// additional one. Note that if you don't have a [`String`], you should use [`Text::from_str`]
	/// or [`Text::from_static_str`].
	///
	/// As [`Text`]s can be embedded, it's more efficient to use a [`Builder`] if you're going to
	/// be created a [`Text`] on the fly.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let fruit = "Apple".to_string();
	/// let text = Text::from_string(fruit);
	/// assert_eq!(*text.as_ref()?, "Apple");
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[must_use]
	pub fn from_string(string: String) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flags(FLAG_FROM_STRING);

		// SAFETY: TODO
		let mut alloc = unsafe { &mut builder.inner_mut().alloc };

		alloc.len = string.len();
		alloc.cap = string.capacity();
		alloc.ptr = std::mem::ManuallyDrop::new(string).as_mut_ptr();

		builder.finish()
	}

	fn is_embedded(&self) -> bool {
		self.flags().contains(FLAG_EMBEDDED)
	}

	fn is_pointer_immutable(&self) -> bool {
		debug_assert!(!self.is_embedded(), "called is_pointer_immutable when embedded");

		self.flags().contains_any(FLAG_NOFREE | FLAG_SHARED)
	}

	fn is_from_string(&self) -> bool {
		debug_assert!(!self.is_embedded(), "called is_from_string when embedded");

		self.flags().contains(FLAG_FROM_STRING)
	}

	/// Gets the length of `self`, in bytes.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let greeting = Text::from_static_str("Hello, world");
	/// assert_eq!(greeting.as_ref()?.len(), 12);
	///
	/// let emoji = Text::from_static_str("😀, 🌎");
	/// assert_eq!(emoji.as_ref()?.len(), 10);
	///
	/// # // ensure allocated things have valid length too
	/// # assert_eq!(Text::from_str(&"hi".repeat(123)).as_ref()?.len(), 246);
	/// # quest::Result::<()>::Ok(())
	/// ```
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

	/// Forcibly sets `self`'s length, in bytes.
	///
	/// # Safety
	/// - `new_len` must be less than or equal to [`capacity()`](Self::capacity)
	/// - The bytes from `old_len..new_len` must be initialized.
	/// - You must [`recalculate_hash` once you're finished updating the string.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// // Allocate enough memory to hold our string.
	/// let text = Text::with_capacity(15);
	/// let mut textmut = text.as_mut()?;
	///
	/// // SAFETY: We know that we have at least 12 bytes of memory
	/// // allocated in the mutable buffer (b/c `with_capacity(15)`),
	/// // so we're  allowed to write to it. Additionally, since we
	/// // wrote to those 12 bytes, they're initialized, so we can
	/// // call `.set_len(12)`. Lastly, we call `recalculate_hash` after.
	/// unsafe {
	///    textmut.as_mut_ptr().copy_from(b"Hello, world".as_ptr(), 12);
	///    textmut.set_len(12);
	/// }
	/// textmut.recalculate_hash();
	///
	/// // Now the data's initialized
	/// assert_eq!(*textmut, "Hello, world");
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub unsafe fn set_len(&mut self, new_len: usize) {
		debug_assert!(
			new_len <= self.capacity(),
			"new len is larger than capacity ({new_len} > {})",
			self.capacity()
		);

		if self.is_embedded() {
			self.set_embedded_len(new_len);
		} else {
			self.inner_mut().alloc.len = new_len;
		}
	}

	pub unsafe fn set_hash(&mut self, hash: u64) {
		self.inner_mut().alloc.hash = hash;
	}

	pub fn recalculate_hash(&mut self) {
		let hash = fast_hash(self.as_str());

		unsafe {
			self.set_hash(hash);
		}
	}

	fn set_embedded_len(&mut self, new_len: usize) {
		debug_assert!(self.is_embedded());

		self.flags().remove_user(EMBED_LENMASK);
		self.flags().insert_user(mask_len(new_len));
	}

	/// Checks to see if `self` has a length of zero bytes.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let empty = Text::from_static_str("");
	/// assert!(empty.as_ref()?.is_empty());
	///
	/// let nonempty = Text::from_static_str("nonempty");
	/// assert!(!nonempty.as_ref()?.is_empty());
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the amount of bytes `self` can hold before reallocating.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::with_capacity(12);
	/// assert!(text.as_ref()?.capacity() >= 12);
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn capacity(&self) -> usize {
		if self.is_embedded() {
			MAX_EMBEDDED_LEN
		} else {
			let inner = self.inner();

			// SAFETY: we know we're allocated, as per the `if`.
			unsafe { inner.alloc.cap }
		}
	}

	pub fn fast_hash(&self) -> u64 {
		// the hash starts at the same offset for both types
		unsafe { self.inner().alloc.hash }
	}

	/// Returns a pointer to the beginning of the `Text` buffer.
	///
	/// You must not write to the pointer; if you need a `*mut u8`, use
	/// [`as_mut_ptr()`](Self::as_mut_ptr) instead.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("Hello");
	/// let ptr = text.as_ref()?.as_ptr();
	///
	/// assert_eq!(unsafe { *ptr }, b'H');
	/// assert_eq!(unsafe { *ptr.offset(4) }, b'o');
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub fn as_ptr(&self) -> *const u8 {
		let inner = self.inner();

		if self.is_embedded() {
			// SAFETY: we know we're embedded, as per the `if`
			unsafe { &inner.embed.buf }.as_ptr()
		} else {
			// SAFETY: we know we're allocated, as per the `if`
			unsafe { inner.alloc.ptr as *const u8 }
		}
	}

	/// Returns a mutable pointer to the beginning of the `Text` buffer.
	///
	/// If the buffer isn't uniquely owned (e.g. `self` was created from [`Text::from_static_str`],
	/// is a [`substr()`](Self::substr), was [`dup`](Self::dup)ed, etc.), this will allocate an
	/// entirely new one and copies the data over.
	///
	/// If you don't need mutable access, use [`as_ptr()`](Self::as_ptr) instead.
	///
	/// # Safety
	/// - You must ensure that the pointer points to a valid `str`.
	/// - After you finish updating things, you must call `recalculate_hash`.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// // Allocate enough memory to hold our string.
	/// let text = Text::with_capacity(15);
	/// let mut textmut = text.as_mut()?;
	///
	/// // SAFETY: We know that we have at least 12 bytes of memory
	/// // allocated in the mutable buffer (b/c `with_capacity(15)`),
	/// // so we're  allowed to write to it. Additionally, since we
	/// // wrote to those 12 bytes, they're initialized, so we can
	/// // call `.set_len(12)`. Lastly, we call `recalculate_hash` after.
	/// unsafe {
	///    textmut.as_mut_ptr().copy_from(b"Hello, world".as_ptr(), 12);
	///    textmut.set_len(12);
	/// }
	/// textmut.recalculate_hash();
	///
	/// // Now the data's initialized
	/// assert_eq!(*textmut, "Hello, world");
	/// # quest::Result::<()>::Ok(())
	/// ```
	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		// If we're embedded, just use a reference to the buffer itself.
		if self.is_embedded() {
			return self.inner_mut().embed.buf.as_mut_ptr();
		}

		if self.is_pointer_immutable() {
			// Both static Rust strings (`FLAG_NOFREE`) and shared strings (`FLAG_SHARED`) don't allow
			// us to write to their pointer. As such, we need to duplicate the `alloc.ptr` field, which
			// gives us ownership of it. Afterwards, we have to remove the relevant flags.
			self.duplicate_alloc_ptr(self.inner().alloc.cap);
		}

		self.inner_mut().alloc.ptr
	}

	/// Returns an immutable slice of bytes from `self`'s internal buffer.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("Hello, 🌎");
	/// assert_eq!(b"Hello, \xF0\x9F\x8C\x8E", text.as_ref()?.as_bytes());
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		// SAFETY: As per the invariants, all bytes from `0..len` must be initialized.
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	/// Returns the internal `str` for `self`.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_static_str("Hello, 🌎");
	/// assert_eq!(text.as_ref()?.as_str(), "Hello, 🌎");
	/// # quest::Result::<()>::Ok(())
	/// ```
	#[inline]
	pub fn as_str(&self) -> &str {
		// SAFETY: An invariant of `Text` is that it's always valid utf8.
		unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
	}

	/// Creates a duplicate of `self` that can be modified independently.
	///
	/// To prevent allocating duplicate buffers, the internal buffer is actually shared across all
	/// clones (and [`substr`](Self::substr)ings). However, once a mutation is done, the buffer will
	/// be copied.
	///
	/// # Examples
	/// ```
	/// # use quest::value::ty::Text;
	/// let text = Text::from_str("Hello, 🌎");
	/// let dup = text.as_ref()?.dup();
	///
	/// assert_eq!(*text.as_ref()?, "Hello, 🌎");
	/// assert_eq!(*dup.as_ref()?, "Hello, 🌎");
	///
	/// text.as_mut()?.push('!');
	/// assert_eq!(*text.as_ref()?, "Hello, 🌎!");
	/// assert_eq!(*dup.as_ref()?, "Hello, 🌎");
	/// # quest::Result::<()>::Ok(())
	/// ```
	// NOTE: currently, if you `dup` then mutably borrow, it'll make a new buffer and ignore the old
	// one. As such, if both the original and `dup`'d ones mutably borrow, then the original buffer
	// will be leaked. This will (hopefully) be fixed with garbage collection, but im not sure.
	#[must_use]
	pub fn dup(&self) -> Gc<Self> {
		if self.is_embedded() {
			// Since we're allocating a new `Self` anyways, we may as well copy over the data.
			return Self::from_str(self.as_str());
		}

		// For allocated strings, you can actually one-for-one copy the body, as we now
		// have `FLAG_SHARED` marked.
		self.flags().insert_user(FLAG_SHARED);

		let mut builder = Self::builder();
		let builder_ptr = builder.inner_mut() as *mut Inner;
		builder.insert_flags(self.flags().get_user());

		// SAFETY: TODO
		unsafe {
			builder_ptr.copy_from_nonoverlapping(self.inner(), 1);
		}

		builder.finish()
	}

	/// Returns a substring of `self` at the given index.
	///
	/// # Panics
	/// This will panic if `idx` is out of bounds of
	#[must_use]
	pub fn substr<I: std::slice::SliceIndex<str, Output = str>>(&self, idx: I) -> Gc<Self> {
		let slice = &self.as_str()[idx];
		self.flags().insert_user(FLAG_SHARED);

		let mut builder = Self::builder();
		builder.insert_flags(FLAG_SHARED);

		builder.inner_mut().alloc = AllocatedText {
			hash: fast_hash(slice),
			ptr: slice.as_ptr() as *mut u8,
			len: slice.len(),
			cap: slice.len(), // capacity = length
		};

		builder.finish()
	}

	unsafe fn duplicate_alloc_ptr(&mut self, capacity: usize) {
		debug_assert!(self.is_pointer_immutable() || self.is_from_string());

		let mut alloc = &mut self.inner_mut().alloc;
		let old_ptr = alloc.ptr;
		let old_cap = alloc.cap;
		let len = alloc.len;

		alloc.ptr = crate::alloc(alloc_ptr_layout(capacity)).as_ptr();
		alloc.ptr.copy_from_nonoverlapping(old_ptr, len);
		alloc.cap = capacity;

		if self.is_from_string() {
			drop(String::from_raw_parts(old_ptr, len, old_cap));
			self.flags().remove_user(FLAG_FROM_STRING);
		}

		self.flags().remove_user(FLAG_NOFREE | FLAG_SHARED);
	}

	/// SAFETY: you must call `recalculate_hash` afterwards
	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
	}

	/// SAFETY: you must call `recalculate_hash` afterwards
	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe { std::str::from_utf8_unchecked_mut(self.as_mut_bytes()) }
	}

	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.is_embedded());

		let new_cap = MAX_EMBEDDED_LEN * 2 + required_len;
		assert!(isize::try_from(new_cap).is_ok(), "too much memory allocated: {new_cap} bytes");

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let hash = self.inner().embed.hash;
			let len = self.embedded_len();
			let ptr = crate::alloc(layout).as_ptr();
			std::ptr::copy(self.inner().embed.buf.as_ptr(), ptr, len);

			self.inner_mut().alloc = AllocatedText { hash, len, cap: new_cap, ptr };

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
		let new_cap = unsafe { self.inner().alloc.cap * 2 } + required_len;
		assert!(isize::try_from(new_cap).is_ok(), "too much memory allocated: {new_cap} bytes");

		// If the pointer is immutable, we have to allocate a new buffer, and then copy over the data.
		if self.is_pointer_immutable() || self.is_from_string() {
			unsafe {
				self.duplicate_alloc_ptr(new_cap);
			}
			return;
		}

		// We have unique ownership of our pointer, so we can `realloc` it without worry.
		unsafe {
			let mut alloc = &mut self.inner_mut().alloc;

			let orig_layout = alloc_ptr_layout(alloc.cap);
			alloc.ptr = crate::realloc(alloc.ptr, orig_layout, new_cap).as_ptr();
			alloc.cap = new_cap;
		}
	}

	fn mut_end_ptr(&mut self) -> *mut u8 {
		unsafe { self.as_mut_ptr().add(self.len()) }
	}

	pub fn push(&mut self, chr: char) {
		let mut buf = [0u8; 4];
		chr.encode_utf8(&mut buf);

		let chrstr = unsafe { std::str::from_utf8_unchecked(&buf[..chr.len_utf8()]) };

		self.push_str(chrstr);
	}

	pub fn push_str(&mut self, string: &str) {
		if self.capacity() <= self.len() + string.len() {
			self.allocate_more(string.len());
		}

		unsafe {
			self.push_str_unchecked(string);
		}

		// Note that we don't call `recalculate_hash` but instead do `acc`, as that
		// is equivalent and faster.
		self.inner_mut().alloc.hash = fast_hash_continue(self.fast_hash(), string);
		self.recalculate_hash();
	}

	// SAFETY: you must recalculate hash afterwards, in addition to other things.
	pub unsafe fn push_str_unchecked(&mut self, string: &str) {
		debug_assert!(
			self.len() + string.len() <= self.capacity(),
			"{} + {} > {}",
			self.len(),
			string.len(),
			self.capacity()
		);

		self.mut_end_ptr().copy_from_nonoverlapping(string.as_ptr(), string.len());

		self.set_len(self.len() + string.len());
	}

	/*
	pub fn unescape(&self) -> Gc<Self> {
		fn needs_unescape(c: char) -> bool {
			!c.is_ascii_graphic() && c != ' '
		}

		 let mut i = 0;

		 'escape_found: loop {
			while i < self.len() {
			  if self[i].needs_escaping() {
				 break 'escape_found; // ie a goto
			  }
			  i += 1;
			}
			return self.dup();
		 }

		 let mut unescaped = Self::allocate(self.len() + 1);
		 self.copy_to(&mut unescaped, i);
		 for chr in self.chars().skip(i) {
			if chr.needs_escaping() {
			  unescape_char(&mut unescaped, chr);
			} else {
			  unescaped.push(chr);
			}
		 }
		 unescaped
	  }
	}*/
}

impl Default for Gc<Text> {
	fn default() -> Self {
		Text::new()
	}
}

impl Hash for Text {
	fn hash<H: Hasher>(&self, h: &mut H) {
		h.write_u64(self.fast_hash());
	}
}

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsMut<str> for Text {
	fn as_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

/*
impl Drop for Text {
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

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.write_str("Text(")?;
		}

		Debug::fmt(self.as_str(), f)?;

		if f.alternate() {
			f.write_str(")")?;
		}

		Ok(())
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl From<&'_ str> for Gc<Text> {
	fn from(string: &str) -> Self {
		Text::from_str(string)
	}
}

impl From<String> for Gc<Text> {
	fn from(string: String) -> Self {
		Text::from_string(string)
	}
}

impl From<&'static str> for Value<Gc<Text>> {
	fn from(text: &'static str) -> Self {
		Text::from_static_str(text).into()
	}
}

impl ToValue for &'static str {
	fn to_value(self) -> Value {
		Value::from(self).to_value()
	}
}

impl ToValue for String {
	fn to_value(self) -> Value {
		Gc::<Text>::from(self).to_value()
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self, rhs) || self.fast_hash() == rhs.fast_hash() && self == rhs.as_str()
	}
}

impl PartialEq<Intern> for Text {
	fn eq(&self, rhs: &Intern) -> bool {
		self.fast_hash() == rhs.fast_hash() && self.as_str() == rhs.as_str()
	}
}

impl PartialEq<str> for Text {
	fn eq(&self, rhs: &str) -> bool {
		self.as_str() == rhs
	}
}

impl PartialEq<&str> for Text {
	fn eq(&self, rhs: &&str) -> bool {
		self == *rhs
	}
}

impl PartialOrd for Text {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for Text {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

impl PartialOrd<str> for Text {
	fn partial_cmp(&self, rhs: &str) -> Option<std::cmp::Ordering> {
		self.as_str().partial_cmp(rhs)
	}
}

pub mod funcs {
	use super::*;

	pub fn concat(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let mut rhs = args[0].to_text()?;

		if text.ptr_eq(rhs) {
			rhs = text.as_ref()?.dup();
		}

		text.as_mut()?.push_str(rhs.as_ref()?.as_str());

		Ok(text.to_value())
	}

	pub fn add(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let rhs = args[0].to_text()?;

		// TODO: allocate a new string
		let text = text.as_ref()?.dup();
		text.as_mut().unwrap().push_str(rhs.as_ref()?.as_str());

		Ok(text.to_value())
	}

	pub fn eql(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if let Some(rhs) = args[0].downcast::<Gc<Text>>() {
			Ok((*text.as_ref()? == *rhs.as_ref()?).to_value())
		} else {
			Ok(false.to_value())
		}
	}

	pub fn len(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;
		Ok((text.as_ref()?.len() as i64).to_value())
	}

	pub fn assign(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let value = args[0];
		let mut frame =
			crate::vm::Frame::with_stackframes(|sfs| *sfs.last().expect("returning from nothing?"))
				.to_value();

		frame.set_attr(text.to_value(), value)?;

		Ok(value)
	}

	pub fn dbg(text: Gc<Text>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(Text::from_string(format!("{:?}", text.as_ref()?.as_str())).to_value())
	}
}

quest_type_attrs! { for Gc<Text>,
	parent Object;
	concat => meth funcs::concat,
	len => meth funcs::len,
	op_eql => meth funcs::eql,
	op_add => meth funcs::add,
	op_assign => meth funcs::assign,
	dbg => meth funcs::dbg,
}

// quest_type! {
// 	#[derive(Debug, NamedType)]
// 	pub struct TextClass(());
// }

// singleton_object! { for TextClass, parentof Gc<Text>, late_binding_parent Object;
// 	"concat" => method!(qs_concat),
// 	"len" => method!(qs_len),
// 	"==" => method!(qs_eql),
// }

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::value::Convertible;
	use crate::Value;

	const JABBERWOCKY: &str = "twas brillig in the slithy tothe did gyre and gimble in the wabe";

	#[test]
	fn test_is_a() {
		assert!(<Gc<Text>>::is_a(Value::from("").to_value()));
		assert!(<Gc<Text>>::is_a(Value::from("x").to_value()));
		assert!(<Gc<Text>>::is_a(Value::from("yesseriie").to_value()));
		assert!(<Gc<Text>>::is_a(Value::from(JABBERWOCKY).to_value()));

		assert!(!<Gc<Text>>::is_a(Value::TRUE.to_value()));
		assert!(!<Gc<Text>>::is_a(Value::FALSE.to_value()));
		assert!(!<Gc<Text>>::is_a(Value::NULL.to_value()));
		assert!(!<Gc<Text>>::is_a(Value::ONE.to_value()));
		assert!(!<Gc<Text>>::is_a(Value::ZERO.to_value()));
		assert!(!<Gc<Text>>::is_a(Value::from(1.0).to_value()));
		assert!(!<Gc<Text>>::is_a(Value::from(RustFn::NOOP).to_value()));
	}

	#[test]
	fn test_get() {
		assert_eq!(*<Gc<Text>>::get(Value::from("")).as_ref().unwrap(), *"");
		assert_eq!(*<Gc<Text>>::get(Value::from("x")).as_ref().unwrap(), *"x");
		assert_eq!(*<Gc<Text>>::get(Value::from("yesseriie")).as_ref().unwrap(), *"yesseriie");
		assert_eq!(*<Gc<Text>>::get(Value::from(JABBERWOCKY)).as_ref().unwrap(), *JABBERWOCKY);
	}

	#[test]
	fn default_is_empty_string() {
		assert_eq!(*Gc::<Text>::default().as_ref().unwrap(), *"");
	}

	#[test]
	fn push_str_updates_hash_correctly() {
		let hash1 = Text::from_str("hello, world").as_ref().unwrap().fast_hash();
		let text = Text::from_str("hello, ");
		text.as_mut().unwrap().push_str("world");
		assert_eq!(text.as_ref().unwrap().fast_hash(), hash1);
	}

	#[test]
	fn larger_strings_still_reallocate_properly() {
		let text = Text::from_string("The time right now in minutes is: ".to_string());

		text.as_mut().unwrap().push_str("4");

		assert_eq!(text.as_ref().unwrap().as_str(), "The time right now in minutes is: 4");
	}
}
