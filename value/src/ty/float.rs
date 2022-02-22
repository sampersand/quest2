pub use crate::{Value, Convertible, Immediate, AnyValue};

pub type Float = f64;

impl From<Float> for Value<Float> {
	#[inline]
	fn from(float: Float) -> Self {
		let bits = (float.to_bits() & !3) | 2;

		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl Convertible for Float {
	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 3) == 2
	}
}

impl Immediate for Float {
	#[inline]
	fn get(value: Value<Self>) -> Self {
		Self::from_bits(value.bits() & !3)
	}
}
