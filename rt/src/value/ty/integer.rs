use crate::value::base::{HasParents, Parents};
use crate::value::ty::{ConvertTo, Text};
use crate::value::{AnyValue, Convertible, Gc, Value};
use crate::vm::Args;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Integer(pub i64);

type Inner = i64;

// all but the top two bits
pub const MAX: Integer = Integer((u64::MAX >> 2) as Inner);
pub const MIN: Integer = Integer(!MAX.0);

impl Value<Integer> {
	pub const ZERO: Self = unsafe { Self::from_bits_unchecked(0b000_001) };
	pub const ONE: Self = unsafe { Self::from_bits_unchecked(0b000_011) };
}

impl From<Integer> for Value<Integer> {
	#[inline]
	fn from(integer: Integer) -> Self {
		let bits = ((integer.0 as u64) << 1) | 1;

		unsafe { Self::from_bits_unchecked(bits) }
	}
}

impl From<Inner> for Value<Integer> {
	#[inline]
	fn from(integer: Inner) -> Self {
		Self::from(Integer(integer))
	}
}

impl PartialEq<Inner> for Integer {
	fn eq(&self, rhs: &Inner) -> bool {
		self.0 == *rhs
	}
}

unsafe impl Convertible for Integer {
	type Output = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 1) == 1
	}

	fn get(value: Value<Self>) -> Self::Output {
		Integer((value.bits() as Inner) >> 1)
	}
}

impl super::AttrConversionDefined for Integer {
	const ATTR_NAME: &'static str = "@int";
}

impl HasParents for Integer {
	unsafe fn init() {
		// todo
	}

	fn parents() -> Parents {
		Default::default() // todo
	}
}

impl ConvertTo<Gc<Text>> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_positional()?;

		let base = if let Ok(base) = args.get("base") {
			args.idx_err_unless(|_| args.len() == 1)?;
			base.convert::<Self>()?.0
		} else {
			args.idx_err_unless(Args::is_empty)?;
			10
		};

		if !(2..=36).contains(&base) {
			Err(format!("invalid radix '{}'", base).into())
		} else {
			Ok(Text::from_string(
				radix_fmt::radix(self.0, base as u8).to_string(),
			))
		}
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
		assert_eq!(Integer::get(Value::from(0)), 0);
		assert_eq!(Integer::get(Value::from(1)), 1);
		assert_eq!(Integer::get(Value::from(-123)), -123);
		assert_eq!(Integer::get(Value::from(14)), 14);
		assert_eq!(Integer::get(Value::from(-1)), -1);
		assert_eq!(Integer::get(Value::from(MIN)), MIN);
		assert_eq!(Integer::get(Value::from(MAX)), MAX);
	}
}
