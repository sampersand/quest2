pub use crate::{Value, Convertible, Immediate, AnyValue};

pub type Boolean = bool;

impl Value<Boolean> {
	pub const TRUE:  Self = unsafe { Self::from_bits_unchecked(0b010_100) };
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b001_100) };
}

impl From<Boolean> for Value<Boolean> {
	fn from(boolean: Boolean) -> Self {
		if boolean {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

unsafe impl Convertible for Boolean {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::TRUE.bits() || value.bits() == Value::FALSE.bits()
	}

	fn get(value: Value<Self>) -> Self::Output {
		value.bits() == Value::TRUE.bits()
	}
}
