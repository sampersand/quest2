use crate::value::{AnyValue, Convertible, Value};

pub type Integer = i64;

pub const MAX: Integer = 4611686018427387903;
pub const MIN: Integer = !MAX;

impl Value<Integer> {
	pub const ZERO: Self = unsafe { Self::from_bits_unchecked(0b000_001) };
	pub const ONE: Self = unsafe { Self::from_bits_unchecked(0b000_011) };
}

impl From<Integer> for Value<Integer> {
	#[inline]
	fn from(integer: Integer) -> Self {
		let bits = ((integer as u64) << 1) | 1;

		unsafe { Self::from_bits_unchecked(bits) }
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

impl crate::value::base::HasParents for Integer {
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
		assert!(Integer::is_a(Value::from(0).any()));
		assert!(Integer::is_a(Value::from(1).any()));
		assert!(Integer::is_a(Value::from(-123).any()));
		assert!(Integer::is_a(Value::from(14).any()));
		assert!(Integer::is_a(Value::from(-1).any()));
		assert!(Integer::is_a(Value::from(MIN).any()));
		assert!(Integer::is_a(Value::from(MAX).any()));

		assert!(!Integer::is_a(Value::TRUE.any()));
		assert!(!Integer::is_a(Value::FALSE.any()));
		assert!(!Integer::is_a(Value::NULL.any()));
		assert!(!Integer::is_a(Value::from(1.0).any()));
		assert!(!Integer::is_a(Value::from("hello").any()));
		assert!(!Integer::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(0, Integer::get(Value::from(0)));
		assert_eq!(1, Integer::get(Value::from(1)));
		assert_eq!(-123, Integer::get(Value::from(-123)));
		assert_eq!(14, Integer::get(Value::from(14)));
		assert_eq!(-1, Integer::get(Value::from(-1)));
		assert_eq!(MIN, Integer::get(Value::from(MIN)));
		assert_eq!(MAX, Integer::get(Value::from(MAX)));
	}
}
