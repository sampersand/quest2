pub use crate::{Value, Convertible, Immediate, AnyValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl Value<Null> {
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b001_100) };
}

impl From<Null> for Value<Null> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl Convertible for Null {
	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::NULL.bits()
	}
}

impl Immediate for Null {
	#[inline]
	fn get(_: Value<Self>) -> Self {
		Self
	}
}
