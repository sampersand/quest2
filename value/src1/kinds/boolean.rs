use crate::{Value, AnyValue};

pub type Boolean = bool;

impl Value<Boolean> {
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b000_100) };
	pub const TRUE: Self  = unsafe { Self::from_bits_unchecked(0b010_100) };
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

unsafe impl crate::Convertible for Boolean {
	fn is_a(value: AnyValue) -> bool {
		value.bits() == Self::FALSE.bits() || value.bits() == Self::TRUE.bits()
	}
}

impl crate::Immediate for Boolean {
	fn get(value: Value<Self>) -> Self {
		value.bits() == Self::TRUE.bits()
	}
}
