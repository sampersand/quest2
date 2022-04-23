use super::{Builder, Text};
use crate::value::gc::Gc;

#[must_use]
pub struct SimpleBuilder(Builder);

impl SimpleBuilder {
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let mut builder = Text::builder();
		builder.allocate_buffer(capacity);
		Self(builder)
	}

	pub fn push_str(&mut self, s: &str) -> &mut Self {
		self.0.text_mut().push_str(s);
		self
	}

	pub fn push(&mut self, c: char) -> &mut Self {
		self.0.text_mut().push(c);
		self
	}

	#[must_use]
	pub fn finish(self) -> Gc<Text> {
		self.0.finish()
	}
}
