use crate::value::{AnyValue, Convertible, Value};

pub type Float = f64;

pub const EPSILON: Float = 0.0000000000000008881784197001252;

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
		(value.bits() & 0b011) == 0b010
	}

	fn get(value: Value<Self>) -> Self::Output {
		Self::from_bits(value.bits() & !3)
	}
}

impl crate::value::base::HasParents for Float {
	unsafe fn init() {
		// todo
	}

	fn parents() -> crate::value::base::Parents {
		Default::default() // todo
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;

	#[test]
	fn test_is_a() {
		assert!(Float::is_a(Value::from(0.0).any()));
		assert!(Float::is_a(Value::from(1.0).any()));
		assert!(Float::is_a(Value::from(-123.456).any()));
		assert!(Float::is_a(Value::from(14.0).any()));
		assert!(Float::is_a(Value::from(f64::NAN).any()));
		assert!(Float::is_a(Value::from(f64::INFINITY).any()));
		assert!(Float::is_a(Value::from(f64::NEG_INFINITY).any()));

		assert!(!Float::is_a(Value::TRUE.any()));
		assert!(!Float::is_a(Value::FALSE.any()));
		assert!(!Float::is_a(Value::NULL.any()));
		assert!(!Float::is_a(Value::ZERO.any()));
		assert!(!Float::is_a(Value::ONE.any()));
		assert!(!Float::is_a(Value::from("hello").any()));
		assert!(!Float::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Float::get(Value::from(0.0)), 0.0);
		assert_eq!(Float::get(Value::from(1.0)), 1.0);
		assert_eq!(
			Float::get(Value::from(-123.456)).to_bits(),
			(-123.456f64).to_bits() & !3
		);
		assert_eq!(
			Float::get(Value::from(14.0)).to_bits(),
			(14.0f64).to_bits() & !3
		);

		let pos_inf = Float::get(Value::from(f64::INFINITY));
		assert!(pos_inf.is_infinite());
		assert!(pos_inf.is_sign_positive());

		let neg_inf = Float::get(Value::from(f64::NEG_INFINITY));
		assert!(neg_inf.is_infinite());
		assert!(neg_inf.is_sign_negative());
	}
}
