use super::{InternalBuilder, List};
use crate::value::gc::Gc;
use crate::{ToValue, Value};

/// A builder for a [`List`]
///
/// This allows for inserting elements without having to call `as_mut` on a [`List`] reference.
///
/// The [`List::with_capacity`] function returns this type.
///
/// # Examples
/// ```
/// use quest::value::ty::{List, Integer};
/// use quest::ToValue;
///
/// let mut builder = List::with_capacity(5);
/// for i in 0..5 {
///    builder.push(Integer::new(i).unwrap().to_value());
/// }
/// let list = builder.finish();
/// // use the `list`
/// # let listref = list.as_ref().unwrap();
/// # for i in 0..5 {
/// #    assert_eq!(i, listref[i].downcast::<Integer>().unwrap().get() as usize);
/// # }
/// ```
#[must_use]
pub struct Builder(InternalBuilder);

impl Default for Builder {
	/// Creates an empty [`Builder`.
	fn default() -> Self {
		Self::new()
	}
}

impl Builder {
	/// Creates a new [`Builder`] with no starting capacity.
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	/// Creates a new [`Builder`] guaranteed to hold at least `capacity` elements.
	///
	/// The function [`List::with_capacity`] is a convenience wrapper around this function.
	pub fn with_capacity(capacity: usize) -> Self {
		let mut builder = InternalBuilder::allocate();

		unsafe {
			builder.allocate_buffer(capacity);
		}

		Self(builder)
	}

	/// Gets how many elements the list can hold before requiring resizing.
	pub fn capacity(&self) -> usize {
		self.0.list().capacity()
	}

	/// Gets how many elements are currently within the list.
	pub fn len(&self) -> usize {
		self.0.list().len()
	}

	/// Adds `ele` to the end of the list.
	pub fn push(&mut self, ele: Value) {
		self.0.list_mut().push(ele);
	}

	/// Adds `ele` to the end of the list, without checking to see if there's enough capacity.
	///
	/// # Safety
	/// You must ensure that `self` has [enough capacity](Self::capacity) to hold `ele`.
	pub unsafe fn push_unchecked(&mut self, ele: Value) {
		self.0.list_mut().push_unchecked(ele)
	}

	/// Concatenates all of `slice` onto the end of `self`.
	pub fn extend_from_slice(&mut self, slice: &[Value]) {
		self.0.list_mut().extend_from_slice(slice);
	}

	/// Concatenates all of `slice` onto the end of `self`, without checking to see if there's enough
	/// capacity.
	///
	/// # Safety
	/// You must ensure that `self` has [enough capacity](Self::capacity) to hold all of `slice`.
	pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[Value]) {
		self.0.list_mut().extend_from_slice(slice)
	}

	/// Finishes the builder, returning the created [`Gc<List>`].
	#[must_use]
	pub fn finish(self) -> Gc<List> {
		unsafe { self.0.finish() }
	}
}

impl Extend<Value> for Builder {
	fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
		self.0.list_mut().extend(iter)
	}
}

impl ToValue for Builder {
	/// [Finishes the builder](Self::finish) and converts the [`Gc<List>`] into a [`Value`].
	fn to_value(self) -> Value {
		self.finish().to_value()
	}
}
