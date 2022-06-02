use crate::value::ty::{ConvertTo, Float, InstanceOf, List, Singleton, Text};
use crate::value::{Convertible, Gc};
use crate::vm::Args;
use crate::{Result, ToValue, Value};

pub type Integer = i64;

// all but the top two bits
pub const MAX: Integer = (u64::MAX >> 2) as Integer;
pub const MIN: Integer = !MAX;

impl crate::value::NamedType for Integer {
	const TYPENAME: crate::value::Typename = "Integer";
}

impl Value<Integer> {
	pub const ZERO: Self = unsafe { Self::from_bits(0b0000_0001) };
	pub const ONE: Self = unsafe { Self::from_bits(0b0000_0011) };
}

impl From<Integer> for Value<Integer> {
	#[inline]
	fn from(integer: Integer) -> Self {
		let bits = ((integer as u64) << 1) | 1;

		unsafe { Self::from_bits(bits) }
	}
}

unsafe impl Convertible for Integer {
	#[inline]
	fn is_a(value: Value) -> bool {
		(value.bits() & 1) == 1
	}

	fn get(value: Value<Self>) -> Self {
		(value.bits() as Self) >> 1
	}
}

impl super::AttrConversionDefined for Integer {
	const ATTR_NAME: crate::value::Intern = crate::value::Intern::at_int;
}

impl ConvertTo<Gc<Text>> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_positional()?;

		let base = if let Some(base) = args.get("base") {
			args.idx_err_unless(|_| args.len() == 1)?;
			base.to_integer()?
		} else {
			args.idx_err_unless(Args::is_empty)?;
			10
		};

		if (2..=36).contains(&base) {
			Ok(Text::from_string(radix_fmt::radix(*self, base as u8).to_string()))
		} else {
			Err(format!("invalid radix '{base}'").into())
		}
	}
}

impl ConvertTo<Float> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		#[allow(clippy::cast_precision_loss)] // Literally the definition of this method.
		Ok(*self as Float)
	}
}

pub mod funcs {
	use super::*;

	pub fn add(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int + args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn sub(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int - args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn mul(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int * args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn div(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let denom = args[0].try_downcast::<Integer>()?;

		if denom == 0 {
			Err("division by zero".to_string().into())
		} else {
			Ok((int / denom).to_value())
		}
	}

	// TODO: verify it's actually modulus
	pub fn r#mod(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let denom = args[0].try_downcast::<Integer>()?;

		if denom == 0 {
			Err("modulo by zero".to_string().into())
		} else {
			Ok((int % denom).to_value())
		}
	}

	pub fn pow(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		#[allow(clippy::cast_precision_loss)] // Eh, maybe in the future i should fix this?
		if let Some(float) = args[0].downcast::<Float>() {
			Ok(((int as Float).powf(float)).to_value())
		} else {
			let exp = args[0].try_downcast::<Integer>()?;

			Ok(int.pow(exp.try_into().expect("todo: exception for not valid number")).to_value())
		}
	}

	pub fn lth(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int < args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn leq(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int <= args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn neg(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((-int).to_value())
	}

	pub fn at_text(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Gc<Text>>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn dbg(int: Integer, args: Args<'_>) -> Result<Value> {
		at_text(int, args)
	}

	// pub fn dbg(val: Value, args: Args<'_>) -> Result<Value> {
	// 	if let Some(int) = val.downcast::<Integer>() {
	// 		dbg_int(int, args)
	// 	} else if val.is_identical(Integer::parent()) {
	// 		args.assert_no_arguments()?;
	// 		Ok(Text::from_string(format!("")).to_value())
	// 	}
	// }

	// TODO: in the future, return an enumerable
	pub fn upto(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let max = args[0].try_downcast::<Integer>()?;

		if max <= int {
			return Ok(List::new().to_value());
		}

		let list = List::with_capacity((max - int) as usize);
		let mut listmut = list.as_mut().unwrap();

		for i in int..=max {
			listmut.push(i.to_value());
		}

		Ok(list.to_value())
	}
}

// impl crate::value::base::HasDefaultParent for Integer {
// 	fn parent() -> Value {}
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IntegerClass;

impl Singleton for IntegerClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Integer", parent Object::instance();
				Intern::op_add => method funcs::add,
				Intern::op_sub => method funcs::sub,
				Intern::op_mul => method funcs::mul,
				Intern::op_div => method funcs::div,
				Intern::op_mod => method funcs::r#mod,
				Intern::op_pow => method funcs::pow,
				Intern::op_lth => method funcs::lth,
				Intern::op_leq => method funcs::leq,
				Intern::op_neg => method funcs::neg,
				Intern::at_text => method funcs::at_text,
				Intern::dbg => method funcs::dbg,
				Intern::upto => method funcs::upto,
			}
		})
	}
}

impl InstanceOf for Integer {
	type Parent = IntegerClass;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::ToValue;

	#[test]
	fn test_is_a() {
		assert!(Integer::is_a(Value::from(0).to_value()));
		assert!(Integer::is_a(Value::from(1).to_value()));
		assert!(Integer::is_a(Value::from(-123).to_value()));
		assert!(Integer::is_a(Value::from(14).to_value()));
		assert!(Integer::is_a(Value::from(-1).to_value()));
		assert!(Integer::is_a(Value::from(MIN).to_value()));
		assert!(Integer::is_a(Value::from(MAX).to_value()));

		assert!(!Integer::is_a(Value::TRUE.to_value()));
		assert!(!Integer::is_a(Value::FALSE.to_value()));
		assert!(!Boolean::is_a(Value::NULL.to_value()));
		assert!(!Integer::is_a(Value::from(1.0).to_value()));
		assert!(!Integer::is_a(Value::from("hello").to_value()));
		assert!(!Integer::is_a(Value::from(RustFn::NOOP).to_value()));
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
		assert!(
			ConvertTo::<Gc<Text>>::convert(&0, Args::new(&[Value::TRUE.to_value()], &[])).is_err()
		);
		assert!(
			ConvertTo::<Gc<Text>>::convert(&0, Args::new(&[], &[("A", Value::TRUE.to_value())]))
				.is_err()
		);
		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[Value::TRUE.to_value()], &[("A", Value::TRUE.to_value())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[Value::TRUE.to_value()], &[("base", Value::from(2).to_value())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(
				&[Value::TRUE.to_value()],
				&[("base", Value::from(2).to_value()), ("A", Value::TRUE.to_value())]
			)
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&0,
			Args::new(&[], &[("base", Value::from(2).to_value()), ("A", Value::TRUE.to_value())])
		)
		.is_err());
	}

	#[test]
	#[ignore]
	fn test_convert_to_text_different_radix() {
		macro_rules! to_text {
			($num:expr, $radix:expr) => {
				ConvertTo::<Gc<Text>>::convert(
					&$num,
					Args::new(&[], &[("base", Value::from($radix as Integer).to_value())]),
				)
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
