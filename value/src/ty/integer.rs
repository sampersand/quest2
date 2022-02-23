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
	type Output = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 1) == 1
	}

	fn get(value: Value<Self>) -> Self::Output {
		(value.bits() as Self) >> 1
	}
}

impl crate::base::HasParents for Integer {
	fn parents() -> crate::base::Parents {
		// TODO
		crate::base::Parents::NONE
	}
}
