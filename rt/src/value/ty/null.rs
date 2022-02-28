use crate::value::{AnyValue, Convertible, Value};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl Value<Null> {
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b011_100) };
}

impl From<Null> for Value<Null> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl Convertible for Null {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::NULL.bits()
	}

	fn get(_: Value<Self>) -> Self::Output {
		Self
	}
}

impl crate::value::base::HasParents for Null {
	fn parents() -> crate::value::base::Parents {
		// TODO
		crate::value::base::Parents::NONE
	}
}

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "null")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;

	#[test]
	fn test_is_a() {
		assert!(Null::is_a(Value::NULL.any()));

		assert!(!Null::is_a(Value::TRUE.any()));
		assert!(!Null::is_a(Value::FALSE.any()));
		assert!(!Null::is_a(Value::ZERO.any()));
		assert!(!Null::is_a(Value::ONE.any()));
		assert!(!Null::is_a(Value::from(1.0).any()));
		assert!(!Null::is_a(Value::from("hello").any()));
		assert!(!Null::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Null, Null::get(Value::from(Null)));
	}
}
