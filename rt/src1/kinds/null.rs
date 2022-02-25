use crate::{Value, AnyValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Null;

impl Value<Null> {
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b001_100) };
}

impl From<Null> for Value<Null> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl crate::Convertible for Null {
	fn is_a(value: AnyValue) -> bool {
		value.bits() == Self::NULL.bits()
	}
}

impl crate::Immediate for Null {
	fn get(_: Value<Self>) -> Self {
		Null
	}
}
