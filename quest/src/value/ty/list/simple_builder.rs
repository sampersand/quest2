use super::{Builder, List};
use crate::value::gc::Gc;
use crate::{ToValue, Value};

#[must_use]
pub struct SimpleBuilder(Builder);

impl Default for SimpleBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl SimpleBuilder {
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let mut builder = List::builder();
		unsafe {
			builder.allocate_buffer(capacity);
		}
		Self(builder)
	}

	pub fn push(&mut self, ele: Value) {
		unsafe { self.0.list_mut() }.push(ele);
	}

	pub unsafe fn push_unchecked(&mut self, ele: Value) {
		self.0.list_mut().push_unchecked(ele)
	}

	pub fn extend_from_slice(&mut self, slice: &[Value]) {
		unsafe { self.0.list_mut() }.extend_from_slice(slice);
	}

	pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[Value]) {
		self.0.list_mut().extend_from_slice(slice)
	}

	#[must_use]
	pub fn finish(self) -> Gc<List> {
		unsafe { self.0.finish() }
	}
}

impl Extend<Value> for SimpleBuilder {
	fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
		unsafe { self.0.list_mut() }.extend(iter)
	}
}

impl ToValue for SimpleBuilder {
	fn to_value(self) -> Value {
		self.finish().to_value()
	}
}
