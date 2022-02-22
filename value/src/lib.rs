extern crate static_assertions as sa;

mod value;
mod base;
mod gc;
pub mod ty;

pub use gc::Gc;
pub use value::{Value, AnyValue};

pub unsafe trait Convertible : Into<Value<Self>> {
	fn is_a(value: AnyValue) -> bool;
	fn downcast(value: AnyValue) -> Option<Value<Self>> {
		if Self::is_a(value) {
			Some(unsafe { std::mem::transmute(value) })
		} else {
			None
		}
	}
}

mod private {
	use super::*;

	pub trait Immediate : Convertible + Copy {
		fn get(value: Value<Self>) -> Self;
	}
}

pub use private::Immediate;
