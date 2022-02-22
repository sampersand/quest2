use crate::{Value, AnyValue};

pub type Integer = i64;

impl From<Integer> for Value<Integer> {
	fn from(int: Integer) -> Self {
		// TODO: debug check if `int` is too large and we're truncating.?

		let bits = ((int as _) << 1) | 1;

		// SAFETY: we always `|` with `2`, so it's never zero. also this is the defn
		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl crate::Convertible for Integer {
	fn is_a(value: AnyValue) -> bool {
		value.bits() & 1 == 1
	}
}

impl crate::Immediate for Integer {
	fn get(value: Value<Self>) -> Self {
		(value.bits() as Self) >> 1
	}
}
