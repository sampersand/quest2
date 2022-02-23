extern crate static_assertions as sa;

mod value;
pub mod base;
mod gc;
pub mod ty;

pub use gc::Gc;
pub use value::{Value, AnyValue};

pub unsafe trait Convertible : Into<Value<Self>> {
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

pub unsafe trait Allocated: Sized + 'static {}
// pub unsafe trait Allocated: Sized + 'static
// where
// 	Gc<Self>: Convertible
// {
// 	fn get(value: Value<Self>) -> Gc<Self> {
// 		Gc::new
// 	}
// }

mod private {
	use super::*;

	// FIXME: people can still access immediates.
	pub trait Immediate : Convertible + Copy {
		fn get(value: Value<Self>) -> Self;
	}
}

pub use private::Immediate;
