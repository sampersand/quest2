use crate::value::base::Flags;
use crate::value::AsAny;
use crate::vm::Args;
use crate::{AnyValue, Value, Result};
use crate::value::gc::{Allocated, Gc};
use std::alloc;
use std::hash::{Hash, Hasher};
use std::fmt::{self, Debug, Display, Formatter};

mod builder;
pub use builder::Builder;

quest_type! {
	#[derive(NamedType)]
	pub struct Text(Inner);
}

impl super::AttrConversionDefined for Gc<Text> {
	const ATTR_NAME: &'static str = "@text";
}

// #[macro_export]
// macro_rules! static_text {
// 	($text:expr) => {
// 		$crate::value::ty::Text($crate::value::base::Base {
// 			header: $crate::value::base::Header {
// 				typeid: ::std::any::TypeId::of::<$crate::value::ty::text::Inner>(),
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
	// TODO: remove pub
	alloc: AllocatedText,
	embed: EmbeddedText,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

#[repr(C)]
#[derive(Clone, Copy)]
struct AllocatedText {
	len: usize,
	cap: usize,
	ptr: *mut u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EmbeddedText {
	buf: [u8; MAX_EMBEDDED_LEN],
}

const MAX_EMBEDDED_LEN: usize = std::mem::size_of::<AllocatedText>();
const FLAG_EMBEDDED: u32 = Flags::USER0;
const FLAG_SHARED: u32 = Flags::USER1;
const FLAG_NOFREE: u32 = Flags::USER2;
const FLAG_FROM_STRING: u32 = Flags::USER3;
const EMBED_LENMASK: u32 = Flags::USER1 | Flags::USER2 | Flags::USER3 | Flags::USER4 | Flags::USER5;

sa::const_assert!(MAX_EMBEDDED_LEN <= unmask_len(EMBED_LENMASK));

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

impl Text {
	const fn inner(&self) -> &Inner {
		self.0.data()
	}

	fn inner_mut(&mut self) -> &mut Inner {
		self.0.data_mut()
	}

	#[must_use]
	pub fn builder() -> Builder {
		Builder::allocate()
	}

	#[must_use]
	pub fn new() -> Gc<Self> {
		Self::with_capacity(0)
	}

	#[must_use]
	pub fn with_capacity(capacity: usize) -> Gc<Self> {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate_buffer(capacity);
			builder.finish() // Nothing else to do, as the default state is valid.
		}
	}

	#[allow(clippy::should_implement_trait)]
	#[must_use]
	pub fn from_str(inp: &str) -> Gc<Self> {
		let mut builder = Self::builder();

		unsafe {
			builder.allocate_buffer(inp.len());
			builder.text_mut().push_str_unchecked(inp);
			builder.finish()
		}
	}

	#[must_use]
	pub fn from_static_str(inp: &'static str) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flag(FLAG_NOFREE);

		unsafe {
			let mut alloc = &mut builder.inner_mut().alloc;

			alloc.ptr = inp.as_ptr() as *mut u8;
			alloc.len = inp.len();
			alloc.cap = alloc.len;

			builder.finish()
		}
	}

	#[must_use]
	pub fn from_string(inp: String) -> Gc<Self> {
		let mut builder = Self::builder();
		builder.insert_flag(FLAG_FROM_STRING);

		unsafe {
			let mut alloc = &mut builder.inner_mut().alloc;

			alloc.ptr = inp.as_ptr() as *mut u8;
			alloc.len = inp.len();
			alloc.cap = inp.capacity();
			std::mem::forget(inp); // so it doesn't become freed

			builder.finish()
		}
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
	/// # use qvm_rt::value::ty::Text;
	/// let greeting = Text::from_static_str("Hello, world");
	/// assert_eq!(greeting.as_ref()?.len(), 12);
	///
	/// let emoji = Text::from_static_str("😀, 🌎");
	/// assert_eq!(emoji.as_ref()?.len(), 10);
	///
	/// # // ensure allocated thigns have valid length too
	/// # assert_eq!(Text::from_str(&"hi".repeat(123)).as_ref()?.len(), 246);
	/// # qvm_rt::Result::<()>::Ok(())
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
	///
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// // Allocate enough memory to hold our string.
	/// let text = Text::with_capacity(15);
	/// let mut textmut = text.as_mut()?;
	///
	/// // SAFETY: We know that we have at least 12 bytes of memory
	/// // allocated in the mutable buffer (b/c `with_capacity(15)`),
	/// // so we're  allowed to write to it. Additionally, since we
	/// // wrote to those 12 bytes, they're initialized, so we can
	/// // call `.set_len(12)`.
	/// unsafe {
	///    textmut.as_mut_ptr().copy_from(b"Hello, world".as_ptr(), 12);
	///    textmut.set_len(12);
	/// }
	///
	/// // Now the data's initialized
	/// assert_eq!(textmut.as_str(), "Hello, world");
	/// # qvm_rt::Result::<()>::Ok(())
	/// ```
	pub unsafe fn set_len(&mut self, new_len: usize) {
		debug_assert!(new_len <= self.capacity(), "new len is larger than capacity");

		if self.is_embedded() {
			self.set_embedded_len(new_len);
		} else {
			self.inner_mut().alloc.len = new_len;
		}
	}

	fn set_embedded_len(&mut self, new_len: usize) {
		debug_assert!(self.is_embedded());
		self.flags().remove(EMBED_LENMASK);
		self.flags().insert(mask_len(new_len));
	}

	/// Checks to see if `self` has a length of zero bytes.
	///
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// let empty = Text::from_static_str("");
	/// assert!(empty.as_ref()?.is_empty());
	///
	/// let nonempty = Text::from_static_str("nonempty");
	/// assert!(!nonempty.as_ref()?.is_empty());
	/// # qvm_rt::Result::<()>::Ok(())
	/// ```
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the amount of bytes `self` can hold before reallocating.
	///
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// let text = Text::with_capacity(12);
	/// assert!(text.as_ref()?.capacity() >= 12);
	/// # qvm_rt::Result::<()>::Ok(())
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

	/// Returns a pointer to the beginning of the `Text` buffer.
	///
	/// You must not write to the pointer; if you need a `*mut u8`, use
	/// [`as_mut_ptr()`](Self::as_mut_ptr) instead.
	///
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// let text = Text::from_static_str("Hello");
	/// let ptr = text.as_ref()?.as_ptr();
	///
	/// assert_eq!(unsafe { *ptr }, b'H');
	/// assert_eq!(unsafe { *ptr.offset(4) }, b'o');
	/// # qvm_rt::Result::<()>::Ok(())
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
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// // Allocate enough memory to hold our string.
	/// let text = Text::with_capacity(15);
	/// let mut textmut = text.as_mut()?;
	///
	/// // SAFETY: We know that we have at least 12 bytes of memory
	/// // allocated in the mutable buffer (b/c `with_capacity(15)`),
	/// // so we're  allowed to write to it. Additionally, since we
	/// // wrote to those 12 bytes, they're initialized, so we can
	/// // call `.set_len(12)`.
	/// unsafe {
	///    textmut.as_mut_ptr().copy_from(b"Hello, world".as_ptr(), 12);
	///    textmut.set_len(12);
	/// }
	///
	/// // Now the data's initialized
	/// assert_eq!(textmut.as_str(), "Hello, world");
	/// # qvm_rt::Result::<()>::Ok(())
	/// ```
	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		if self.is_embedded() {
			return self.inner_mut().embed.buf.as_mut_ptr();
		}

		if self.is_pointer_immutable() {
			// Both static Rust strings (`FLAG_NOFREE`) and shared strings (`FLAG_SHARED`) don't allow
			// us to write to their pointer. As such, we need to duplicate the `alloc.ptr` field, which
			// gives us ownership of it. Afterwards, we have to remove the relevant flags.
			self.duplicate_alloc_ptr(self.inner().alloc.len);
		}

		self.inner_mut().alloc.ptr
	}

	/// Returns an immutable slice of bytes from `self`'s internal buffer.
	///
	/// # Examples
	/// ```
	/// # use qvm_rt::value::ty::Text;
	/// let text = Text::from_static_str("Hello, 🌎");
	/// assert_eq!(b"Hello, \xF0\x9F\x8C\x8E", text.as_ref()?.as_bytes());
	/// # qvm_rt::Result::<()>::Ok(())
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
	/// # use qvm_rt::value::ty::Text;
	/// let text = Text::from_static_str("Hello, 🌎");
	/// assert_eq!(text.as_ref()?.as_str(), "Hello, 🌎");
	/// # qvm_rt::Result::<()>::Ok(())
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
	/// # use qvm_rt::value::ty::Text;
	/// let text = Text::from_str("Hello, 🌎");
	/// let dup = text.as_ref()?.dup();
	///
	/// assert_eq!(text.as_ref()?.as_str(), "Hello, 🌎");
	/// assert_eq!(dup.as_ref()?.as_str(), "Hello, 🌎");
	///
	/// text.as_mut()?.push('!');
	/// assert_eq!(text.as_ref()?.as_str(), "Hello, 🌎!");
	/// assert_eq!(dup.as_ref()?.as_str(), "Hello, 🌎");
	/// # qvm_rt::Result::<()>::Ok(())
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
		self.flags().insert(FLAG_SHARED);

		// SAFETY: TODO
		unsafe {
			let mut builder = Self::builder();
			let builder_ptr = builder.inner_mut() as *mut Inner;
			builder_ptr.copy_from_nonoverlapping(self.inner() as *const Inner, 1);
			builder.insert_flag(self.flags().get());
			builder.finish()
		}
	}

	/// Returns a substring of `self` at the given index.
	///
	/// # Panics
	/// This will panic if `idx` is out of bounds of
	#[must_use]
	pub fn substr<I: std::slice::SliceIndex<str, Output = str>>(&self, idx: I) -> Gc<Self> {
		let slice = &self.as_str()[idx];

		unsafe {
			self.flags().insert(FLAG_SHARED);

			let mut builder = Self::builder();
			builder.insert_flag(FLAG_SHARED);
			builder.inner_mut().alloc = AllocatedText {
				ptr: slice.as_ptr() as *mut u8,
				len: slice.len(),
				cap: slice.len(), // capacity = length
			};

			builder.finish()
		}
	}

	unsafe fn duplicate_alloc_ptr(&mut self, capacity: usize) {
		debug_assert!(self.is_pointer_immutable());

		let mut alloc = &mut self.inner_mut().alloc;
		let old_ptr = alloc.ptr;
		let old_cap = alloc.cap;
		let len = alloc.len;
		alloc.ptr = crate::alloc(alloc_ptr_layout(capacity));
		alloc.cap = capacity;
		std::ptr::copy(old_ptr, alloc.ptr, alloc.len);

		if self.is_from_string() {
			drop(String::from_raw_parts(old_ptr, len, old_cap));
			self.flags().remove(FLAG_FROM_STRING);
		}

		self.flags().remove(FLAG_NOFREE | FLAG_SHARED);
	}

	pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
		std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
	}

	pub fn as_mut_str(&mut self) -> &mut str {
		unsafe { std::str::from_utf8_unchecked_mut(self.as_mut_bytes()) }
	}

	fn allocate_more_embeded(&mut self, required_len: usize) {
		debug_assert!(self.is_embedded());

		let new_cap = std::cmp::max(MAX_EMBEDDED_LEN * 2, required_len);
		assert!(new_cap <= isize::MAX as usize, "too much memory allocated");

		let layout = alloc_ptr_layout(new_cap);

		unsafe {
			let len = self.embedded_len();
			let ptr = crate::alloc(layout);
			std::ptr::copy(self.inner().embed.buf.as_ptr(), ptr, len);

			self.inner_mut().alloc = AllocatedText {
				len,
				cap: new_cap,
				ptr,
			};

			self.flags().remove(FLAG_EMBEDDED | EMBED_LENMASK);
		}
	}

	fn allocate_more(&mut self, required_len: usize) {
		// If we're allocating more, and we're embedded, then we are going to need to allocate an
		// entirely new buffer in memory, and no longer be embedded.
		if self.is_embedded() {
			return self.allocate_more_embeded(required_len);
		}

		// Find the new capacity we'll need.
		let new_cap = std::cmp::max(unsafe { self.inner().alloc.cap } * 2, required_len);
		assert!(new_cap <= isize::MAX as usize, "too much memory allocated");

		// If the pointer is immutable, we have to allocate a new buffer, and then copy over the data.
		if self.is_pointer_immutable() || self.is_from_string() {
			unsafe {
				self.duplicate_alloc_ptr(new_cap);
			}
			return;
		}

		dbg!(&self);

		// We have unique ownership of our pointer, so we can `realloc` it without worry.
		unsafe {
			let mut alloc = &mut self.inner_mut().alloc;

			let orig_layout = alloc_ptr_layout(alloc.cap);
			alloc.ptr = crate::realloc(alloc.ptr, orig_layout, new_cap);
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
	}

	pub unsafe fn push_str_unchecked(&mut self, string: &str) {
		debug_assert!(self.capacity() >= self.len() + string.len());

		std::ptr::copy(string.as_ptr(), self.mut_end_ptr(), string.len());
		self.set_len(self.len() + string.len());
	}
}

impl Default for Gc<Text> {
	fn default() -> Self {
		Text::new()
	}
}

impl Hash for Text {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.as_str().hash(h)
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

impl AsAny for &'static str {
	fn as_any(self) -> AnyValue {
		Value::from(self).any()
	}
}

impl AsAny for String {
	fn as_any(self) -> AnyValue {
		Gc::<Text>::from(self).as_any()
	}
}

impl Gc<Text> {
	pub fn qs_concat(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let mut rhs = args[0].to_text()?;

		if self.ptr_eq(rhs) {
			rhs = self.as_ref()?.dup();
		}

		self.as_mut()?.push_str(rhs.as_ref()?.as_str());

		Ok(self.as_any())
	}

	pub fn qs_eql(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if let Some(text) = args[0].downcast::<Self>() {
			Ok((*self.as_ref()? == *text.as_ref()?).as_any())
		} else {
			Ok(false.as_any())
		}
	}

	pub fn qs_len(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;
		Ok((self.as_ref()?.len() as i64).as_any())
	}
}

quest_type_attrs! { for Gc<Text>,
	late_binding_parent Object;
	"concat" => meth Gc::<Text>::qs_concat,
	"len" => meth Gc::<Text>::qs_len,
	"==" => meth Gc::<Text>::qs_eql
}

impl Eq for Text {}
impl PartialEq for Text {
	fn eq(&self, rhs: &Self) -> bool {
		self == rhs.as_str()
	}
}

impl PartialEq<str> for Text {
	fn eq(&self, rhs: &str) -> bool {
		self.as_str() == rhs
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::value::Convertible;
	use crate::Value;

	const JABBERWOCKY: &str = "twas brillig in the slithy tothe did gyre and gimble in the wabe";

	#[test]
	fn test_is_a() {
		assert!(<Gc<Text>>::is_a(Value::from("").any()));
		assert!(<Gc<Text>>::is_a(Value::from("x").any()));
		assert!(<Gc<Text>>::is_a(Value::from("yesseriie").any()));
		assert!(<Gc<Text>>::is_a(Value::from(JABBERWOCKY).any()));

		assert!(!<Gc<Text>>::is_a(Value::TRUE.any()));
		assert!(!<Gc<Text>>::is_a(Value::FALSE.any()));
		assert!(!<Gc<Text>>::is_a(Default::default()));
		assert!(!<Gc<Text>>::is_a(Value::ONE.any()));
		assert!(!<Gc<Text>>::is_a(Value::ZERO.any()));
		assert!(!<Gc<Text>>::is_a(Value::from(1.0).any()));
		assert!(!<Gc<Text>>::is_a(Value::from(RustFn::NOOP).any()));
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
}
