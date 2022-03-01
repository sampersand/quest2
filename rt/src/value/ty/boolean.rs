use crate::value::{AnyValue, Convertible, Value};

pub type Boolean = bool;

impl Value<Boolean> {
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b001_100) };
	pub const TRUE: Self = unsafe { Self::from_bits_unchecked(0b010_100) };
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

impl crate::value::base::HasParents for Boolean {
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
		assert!(Boolean::is_a(Value::FALSE.any()));
		assert!(Boolean::is_a(Value::TRUE.any()));

		assert!(!Boolean::is_a(Value::NULL.any()));
		assert!(!Boolean::is_a(Value::ZERO.any()));
		assert!(!Boolean::is_a(Value::ONE.any()));
		assert!(!Boolean::is_a(Value::from(12.0).any()));
		assert!(!Boolean::is_a(Value::from("hello").any()));
		assert!(!Boolean::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Boolean::get(Value::FALSE), false);
		assert_eq!(Boolean::get(Value::TRUE), true);
	}
}
