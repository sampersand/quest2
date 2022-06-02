#[macro_use]
pub mod ty;

pub mod base;
pub mod gc;
mod intern;
mod value;

pub use gc::Gc;
pub use intern::Intern;
pub use value::Value;

pub trait HasDefaultParent {
	fn parent() -> Value;
}

pub type Typename = &'static str;
pub trait NamedType {
	const TYPENAME: Typename;
}

pub unsafe trait Convertible: Into<Value<Self>> {
	fn is_a(value: Value) -> bool;

	#[must_use]
	fn downcast(value: Value) -> Option<Value<Self>> {
		if Self::is_a(value) {
			Some(unsafe { std::mem::transmute(value) })
		} else {
			None
		}
	}

	fn get(value: Value<Self>) -> Self;
}

pub trait ToValue {
	fn to_value(self) -> Value;
}

impl<T: Convertible> ToValue for T {
	fn to_value(self) -> Value {
		self.into().to_value()
	}
}
