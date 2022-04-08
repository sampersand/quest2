use super::{Text, Builder};
use crate::value::gc::Gc;

#[must_use]
pub struct SimpleBuilder(Builder);

impl SimpleBuilder {
	pub fn new() -> Self {
		let mut builder = Text::builder();
		unsafe { 
			builder.allocate_buffer(0);
		}
		Self(builder)
	}

	pub fn push_str(&mut self, s: &str) -> &mut Self {
		unsafe { self.0.text_mut() }.push_str(s);
		self
	}

	pub fn push(&mut self, c: char) -> &mut Self {
		unsafe { self.0.text_mut() }.push(c);
		self
	}

	#[must_use]
	pub fn finish(self) -> Gc<Text> {
		unsafe { self.0.finish() }
	}
}
