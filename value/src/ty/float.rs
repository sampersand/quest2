pub use crate::{AnyValue, Convertible, Immediate, Value};

pub type Float = f64;

impl From<Float> for Value<Float> {
	#[inline]
	fn from(float: Float) -> Self {
		let bits = (float.to_bits() & !3) | 2;

		unsafe { Self::from_bits_unchecked(bits) }
	}
}

unsafe impl Convertible for Float {
	type Output = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 3) == 2
	}

	fn get(value: Value<Self>) -> Self::Output {
		Self::from_bits(value.bits() & !3)
	}
}

impl crate::base::HasParents for Float {
	fn parents() -> crate::base::Parents {
		// TODO
		crate::base::Parents::NONE
	}
}
