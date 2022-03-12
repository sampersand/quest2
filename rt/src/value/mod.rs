#![allow(
	clippy::unusual_byte_groupings // value representations use the last three bits, not four.
)]

#[macro_use]
pub mod ty;

pub mod base;
pub mod gc;
mod value;

pub use gc::Gc;
pub use value::{AnyValue, Value};

pub trait NamedType {
	const TYPENAME: &'static str;
}

pub unsafe trait Convertible: Into<Value<Self>> {
	fn is_a(value: AnyValue) -> bool;

	#[must_use]
	fn downcast(value: AnyValue) -> Option<Value<Self>> {
		if Self::is_a(value) {
			Some(unsafe { std::mem::transmute(value) })
		} else {
			None
		}
	}

	fn get(value: Value<Self>) -> Self;
}

pub trait AsAny {
	fn as_any(self) -> AnyValue;
}

impl<T: Convertible> AsAny for T {
	fn as_any(self) -> AnyValue {
		self.into().any()
	}
}

