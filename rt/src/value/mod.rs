#[macro_use]
pub mod ty;

pub mod base;
mod gc;
mod value;

pub use gc::Gc;
pub use value::{AnyValue, Value};

pub unsafe trait Convertible: Into<Value<Self>> {
	type Output: std::fmt::Debug;

	fn is_a(value: AnyValue) -> bool;
	fn downcast(value: AnyValue) -> Option<Value<Self>> {
		if Self::is_a(value) {
			Some(unsafe { std::mem::transmute(value) })
		} else {
			None
		}
	}

	fn get(value: Value<Self>) -> Self::Output;
}
