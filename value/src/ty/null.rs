pub use crate::{AnyValue, Convertible, Value};
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

impl crate::base::HasParents for Null {
	fn parents() -> crate::base::Parents {
		// TODO
		crate::base::Parents::NONE
	}
}

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "null")
	}
}
