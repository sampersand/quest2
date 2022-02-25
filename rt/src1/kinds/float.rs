use crate::{Value, AnyValue};

pub type Float = f64;

impl From<Float> for Value<Float> {
	fn from(float: Float) -> Self {
		// TODO: debug check if `float` is too large and we're truncating.

		let bits = (float.to_bits() & !3) | 2;

		// SAFETY: we always `|` with `2`, so it's never zero. also this is the defn
		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl crate::Convertible for Float {
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 3) == 2
	}
}

impl crate::Immediate for Float {
	fn get(value: Value<Self>) -> Self {
		Self::from_bits(value.bits() & !2)
	}
}
