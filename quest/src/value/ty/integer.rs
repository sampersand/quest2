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

	pub fn op_add(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int + args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_sub(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int - args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_mul(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int * args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_div(int: Integer, args: Args<'_>) -> Result<Value> {
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
	pub fn op_mod(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let denom = args[0].try_downcast::<Integer>()?;

		if denom == 0 {
			Err("modulo by zero".to_string().into())
		} else {
			Ok((int % denom).to_value())
		}
	}

	pub fn op_pow(int: Integer, args: Args<'_>) -> Result<Value> {
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

	pub fn op_lth(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int < args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_leq(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int <= args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_gth(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int > args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_geq(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int >= args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_cmp(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.cmp(&args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_neg(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((-int).to_value())
	}

	pub fn op_shl(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int << args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_shr(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int >> args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_bitand(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int & args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_bitor(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int | args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_bitxor(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int ^ args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_bitneg(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((!int).to_value())
	}

	pub fn at_text(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Gc<Text>>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn at_float(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Float>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn at_int(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Integer>::convert(&int, args).map(ToValue::to_value)
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

		if max < int {
			return Ok(List::new().to_value());
		}

		let list = List::with_capacity((max - int) as usize);
		let mut listmut = list.as_mut().unwrap();

		for i in int..=max {
			listmut.push(i.to_value());
		}

		Ok(list.to_value())
	}

	pub fn downto(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let min = args[0].try_downcast::<Integer>()?;

		if min > int {
			return Ok(List::new().to_value());
		}

		let list = List::with_capacity((int - min) as usize);
		let mut listmut = list.as_mut().unwrap();

		for i in (min..=int).rev() {
			listmut.push(i.to_value());
		}

		Ok(list.to_value())
	}

	pub fn times(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		upto(0, Args::new(&[(int - 1).to_value()], &[]))
	}

	pub fn chr(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		if let Some(chr) = u32::try_from(int).ok().and_then(char::from_u32) {
			let mut builder = Text::simple_builder();
			builder.push(chr);
			Ok(builder.finish().to_value())
		} else {
			Err(format!("oops, number {int:x} is out of bounds!").into())
		}
	}

	pub fn is_even(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int % 2 == 0).to_value())
	}

	pub fn is_odd(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int % 2 == 1).to_value())
	}

	pub fn is_zero(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int == 0).to_value())
	}

	pub fn is_one(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int == 1).to_value())
	}

	pub fn is_positive(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int > 0).to_value())
	}

	pub fn is_negative(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int < 0).to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IntegerClass;

impl Singleton for IntegerClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Integer", parent Object::instance();
				Intern::op_neg => method funcs::op_neg,
				Intern::op_add => method funcs::op_add,
				Intern::op_sub => method funcs::op_sub,
				Intern::op_mul => method funcs::op_mul,
				Intern::op_div => method funcs::op_div,
				Intern::op_mod => method funcs::op_mod,
				Intern::op_pow => method funcs::op_pow,

				Intern::op_lth => method funcs::op_lth,
				Intern::op_leq => method funcs::op_leq,
				Intern::op_gth => method funcs::op_gth,
				Intern::op_geq => method funcs::op_geq,
				Intern::op_cmp => method funcs::op_cmp,

				Intern::op_shl => method funcs::op_shl,
				Intern::op_shr => method funcs::op_shr,
				Intern::op_bitand => method funcs::op_bitand,
				Intern::op_bitor => method funcs::op_bitor,
				Intern::op_bitxor => method funcs::op_bitxor,
				Intern::op_bitneg => method funcs::op_bitneg,

				Intern::times => method funcs::times,
				Intern::upto => method funcs::upto,
				Intern::downto => method funcs::downto,

				Intern::is_even => method funcs::is_even,
				Intern::is_odd => method funcs::is_odd,
				Intern::is_zero => method funcs::is_zero,
				Intern::is_positive => method funcs::is_positive,
				Intern::is_negative => method funcs::is_negative,

				Intern::chr => method funcs::chr,
				Intern::at_text => method funcs::at_text,
				Intern::at_float => method funcs::at_float,
				Intern::at_int => method funcs::at_int,
				Intern::dbg => method funcs::dbg,
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

	#[test]
	fn op_neg() {
		// println!("{} {} {} {}", -i8::MAX, i8::MAX, i8::MIN, 0);
		// println!("{MAX} {:?}", (-MAX).to_value().downcast::<Integer>().unwrap());
		// println!("{MIN} {:?}", (-MIN).to_value().downcast::<Integer>().unwrap());

		assert_code! {
			r#"
				# Make sure we aren't using a `-` prefix when parsing
				assert( -(1) == (0 - 1) );
				assert( -(0) == 0 );
				assert( -(-(1)) == 1 );
				#assert( -({MAX}) == {MIN} );
				#assert( -({MIN}) == {MAX} );
			"#,
		}
	}

	/*
				create_class! { "Integer", parent Object::instance();
					Intern::op_neg => method funcs::op_neg,
					Intern::op_add => method funcs::op_add,
					Intern::op_sub => method funcs::op_sub,
					Intern::op_mul => method funcs::op_mul,
					Intern::op_div => method funcs::op_div,
					Intern::op_mod => method funcs::op_mod,
					Intern::op_pow => method funcs::op_pow,

					Intern::op_lth => method funcs::op_lth,
					Intern::op_leq => method funcs::op_leq,
					Intern::op_gth => method funcs::op_gth,
					Intern::op_geq => method funcs::op_geq,
					Intern::op_cmp => method funcs::op_cmp,

					Intern::op_shl => method funcs::op_shl,
					Intern::op_shr => method funcs::op_shr,
					Intern::op_bitand => method funcs::op_bitand,
					Intern::op_bitor => method funcs::op_bitor,
					Intern::op_bitxor => method funcs::op_bitxor,
					Intern::op_bitneg => method funcs::op_bitneg,

					Intern::times => method funcs::times,
					Intern::upto => method funcs::upto,
					Intern::downto => method funcs::downto,

					Intern::is_even => method funcs::is_even,
					Intern::is_odd => method funcs::is_odd,
					Intern::is_zero => method funcs::is_zero,
					Intern::is_positive => method funcs::is_positive,
					Intern::is_negative => method funcs::is_negative,

					Intern::chr => method funcs::chr,
					Intern::at_text => method funcs::at_text,
					Intern::at_float => method funcs::at_float,
					Intern::at_int => method funcs::at_int,
					Intern::dbg => method funcs::dbg,
				}
			})
	*/
}
