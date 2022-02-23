pub use crate::{Value, Convertible, Immediate, AnyValue};

pub type Integer = i64;

impl Value<Integer> {
	pub const ZERO: Self = unsafe { Self::from_bits_unchecked(0b000_001) };
	pub const ONE:  Self = unsafe { Self::from_bits_unchecked(0b000_011) };
}

impl From<Integer> for Value<Integer> {
	#[inline]
	fn from(integer: Integer) -> Self {
		let bits = ((integer as u64) << 1) | 1;

		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl Convertible for Integer {
	type Inner = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 1) == 1
	}
}

impl Immediate for Integer {
	#[inline]
	fn get(value: Value<Self>) -> Self {
		(value.bits() as Self) >> 1
	}
}
