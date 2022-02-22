#[macro_use]
extern crate static_assertions;


pub use value::{Value, AnyValue};
use base::Allocated;

mod gc;
pub use gc::Gc;

mod err;
pub use err::{Error, Result};

mod attr;
pub use attr::Attributes;
pub mod base;
pub mod kinds;
pub mod value;

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

pub trait Immediate : Convertible + Copy {
	fn get(value: Value<Self>) -> Self;
}

pub trait Allocation : Convertible {
	fn get(value: &Value<Self>) -> &Self;
}

	// + Into<Value<Self>>
	// fn parents(me: &Value<Self>) -> &[AnyValue]
	// 	where Self: Sized;

	// fn unique_id(&self) -> u64;
	// fn attrs(&self) -> &Attributes;
