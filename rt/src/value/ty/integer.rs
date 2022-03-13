use crate::value::ty::{ConvertTo, Float, Text};
use crate::value::{AnyValue, Convertible, Gc, Value, AsAny};
use crate::vm::Args;
use crate::Result;

pub type Integer = i64;

// all but the top two bits
pub const MAX: Integer = (u64::MAX >> 2) as Integer;
pub const MIN: Integer = !MAX;

impl crate::value::NamedType for Integer {
	const TYPENAME: &'static str = "Integer";
}

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
	#[inline]
	fn is_a(value: AnyValue) -> bool {
		(value.bits() & 1) == 1
	}

	fn get(value: Value<Self>) -> Self {
		(value.bits() as Self) >> 1
	}
}

impl super::AttrConversionDefined for Integer {
	const ATTR_NAME: &'static str = "@int";
}

pub trait IntegerExt : Sized {
	fn qs_add(self, args: Args<'_>) -> Result<AnyValue>;
	fn qs_at_text(self, args: Args<'_>) -> Result<AnyValue>;
}

impl IntegerExt for Integer {
	fn qs_add(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((self + args[0].to_integer()?).as_any())
	}

	fn qs_at_text(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Gc<Text>>::convert(&self, args).map(AsAny::as_any)
	}
}

quest_type_attrs! { for Integer, parent Object;
	"+" => func method!(Integer::qs_add),
	"@text" => func method!(Integer::qs_at_text),
}

impl ConvertTo<Gc<Text>> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_positional()?;

		let base = if let Ok(base) = args.get("base") {
			args.idx_err_unless(|_| args.len() == 1)?;
			base.to_integer()?
		} else {
			args.idx_err_unless(Args::is_empty)?;
			10
		};

		if !(2..=36).contains(&base) {
			Err(format!("invalid radix '{}'", base).into())
		} else {
			Ok(Text::from_string(radix_fmt::radix(*self, base as u8).to_string()))
		}
	}
}

impl ConvertTo<Float> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		Ok(*self as Float)
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
		assert!(!Integer::is_a(Default::default()));
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

	#[test]
	fn test_convert_to_float() {
		// TODO: how do we want to test conversions
	}

	#[test]
	fn test_convert_to_text_noargs() {
		macro_rules! to_text {
			($num:expr) => {
				ConvertTo::<Gc<Text>>::convert(&$num, Args::default())
					.unwrap()
					.as_ref()
					.unwrap()
					.as_str()
			};
		}

		assert_eq!("0", to_text!(0));
		assert_eq!("1", to_text!(1));
		assert_eq!("-123", to_text!(-123));
		assert_eq!("14", to_text!(14));
		assert_eq!("-1", to_text!(-1));
		assert_eq!(MIN.to_string(), to_text!(MIN));
		assert_eq!(MAX.to_string(), to_text!(MAX));
	}

	#[test]
	fn test_convert_to_text_bad_args_error() {
		assert!(ConvertTo::<Gc<Text>>::convert(&0, Args::new(&[Value::TRUE.any()], &[])).is_err());
		assert!(
			ConvertTo::<Gc<Text>>::convert(&0, Args::new(&[], &[("A", Value::TRUE.any())])).is_err()
		);
		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[Value::TRUE.any()], &[("A", Value::TRUE.any())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[Value::TRUE.any()], &[("base", Value::from(2).any())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[Value::TRUE.any()], &[("base", Value::from(2).any()), ("A", Value::TRUE.any())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[], &[("base", Value::from(2).any()), ("A", Value::TRUE.any())])
		)
		.is_err());
	}

	#[test]
	fn test_convert_to_text_different_radix() {
		macro_rules! to_text {
			($num:expr, $radix:expr) => {
				ConvertTo::<Gc<Text>>::convert(&$num, Args::new(&[], &[("base", Value::from($radix as Integer).any())]))
					.unwrap()
					.as_ref()
					.unwrap()
					.as_str()
			};
		}

		for radix in 2..=36 {
			assert_eq!(radix_fmt::radix(0 as Integer, radix).to_string(), to_text!(0, radix));
			assert_eq!(radix_fmt::radix(1 as Integer, radix).to_string(), to_text!(1, radix));
			assert_eq!(radix_fmt::radix(-123 as Integer, radix).to_string(), to_text!(-123, radix));
			assert_eq!(radix_fmt::radix(14 as Integer, radix).to_string(), to_text!(14, radix));
			assert_eq!(radix_fmt::radix(-1 as Integer, radix).to_string(), to_text!(-1, radix));
			assert_eq!(radix_fmt::radix(MIN, radix).to_string(), to_text!(MIN, radix));
			assert_eq!(radix_fmt::radix(MAX, radix).to_string(), to_text!(MAX, radix));
		}
	}
}
