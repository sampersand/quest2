use crate::value::ty::{ConvertTo, InstanceOf, Integer, Singleton, Text};
use crate::value::{Convertible, Gc};
use crate::vm::Args;
use crate::{Result, ToValue, Value};

pub type Float = f64;

pub const EPSILON: Float = 0.0000000000000008881784197001252;

impl From<Float> for Value<Float> {
	#[inline]
	fn from(float: Float) -> Self {
		let bits = (float.to_bits() & !3) | 2;

		unsafe { Self::from_bits(bits) }
	}
}

impl crate::value::NamedType for Float {
	const TYPENAME: crate::value::Typename = "Float";
}

unsafe impl Convertible for Float {
	#[inline]
	fn is_a(value: Value) -> bool {
		(value.bits() & 0b011) == 0b010
	}

	fn get(value: Value<Self>) -> Self {
		Self::from_bits(value.bits() & !3)
	}
}

impl ConvertTo<Gc<Text>> for Float {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_arguments()?;

		Ok(Text::from_string(self.to_string()))
	}
}

impl ConvertTo<Integer> for Float {
	fn convert(&self, args: Args<'_>) -> Result<Integer> {
		args.assert_no_arguments()?;

		Ok(Integer::new_truncate(*self as _))
	}
}

pub mod funcs {
	use super::*;

	pub fn to_float(float: Float, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Float>::convert(&float, args).map(ToValue::to_value)
	}

	pub fn to_text(float: Float, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Gc<Text>>::convert(&float, args).map(ToValue::to_value)
	}

	pub fn to_int(float: Float, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Integer>::convert(&float, args).map(ToValue::to_value)
	}

	pub fn dbg(float: Float, args: Args<'_>) -> Result<Value> {
		to_text(float, args)
	}

	pub fn op_neg(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((-float).to_value())
	}

	pub fn op_add(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float + args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_sub(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float - args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_mul(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float * args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_div(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float / args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_mod(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float % args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_pow(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(float.powf(args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_lth(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float < args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_leq(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float <= args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_gth(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float > args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_geq(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((float >= args[0].try_downcast::<Float>()?).to_value())
	}

	pub fn op_cmp(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(float
			.partial_cmp(&args[0].try_downcast::<Float>()?)
			.map(|x| x.to_value())
			.unwrap_or_default())
	}

	pub fn is_zero(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((float == 0.0).to_value())
	}

	pub fn is_positive(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((float > 0.0).to_value())
	}

	pub fn is_negative(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((float < 0.0).to_value())
	}

	pub fn is_whole(float: Float, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		// TODO: this wont work for things outside the representable range of `Integer`.
		Ok((float as i64 as Float == float).to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FloatClass;

impl Singleton for FloatClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Float", parent Object::instance();
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

				Intern::is_zero => method funcs::is_zero,
				Intern::is_positive => method funcs::is_positive,
				Intern::is_negative => method funcs::is_negative,
				Intern::is_whole => method funcs::is_whole,

				Intern::to_text => method funcs::to_text,
				Intern::to_float => method funcs::to_float,
				Intern::to_int => method funcs::to_int,

				Intern::dbg => method funcs::dbg,
			}
		})
	}
}

impl InstanceOf for Float {
	type Parent = FloatClass;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::ToValue;

	#[test]
	fn test_is_a() {
		assert!(Float::is_a(0.0.to_value()));
		assert!(Float::is_a(1.0.to_value()));
		assert!(Float::is_a((-123.456).to_value()));
		assert!(Float::is_a(14.0.to_value()));
		assert!(Float::is_a(f64::NAN.to_value()));
		assert!(Float::is_a(f64::INFINITY.to_value()));
		assert!(Float::is_a(f64::NEG_INFINITY.to_value()));

		assert!(!Float::is_a(Value::TRUE.to_value()));
		assert!(!Float::is_a(Value::FALSE.to_value()));
		assert!(!Boolean::is_a(Value::NULL.to_value()));
		assert!(!Float::is_a(Value::ZERO.to_value()));
		assert!(!Float::is_a(Value::ONE.to_value()));
		assert!(!Float::is_a("hello".to_value()));
		assert!(!Float::is_a(RustFn::NOOP.to_value()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Float::get(Value::from(0.0)), 0.0);
		assert_eq!(Float::get(Value::from(1.0)), 1.0);
		assert_eq!(Float::get(Value::from(-123.456)).to_bits(), (-123.456f64).to_bits() & !3);
		assert_eq!(Float::get(Value::from(14.0)).to_bits(), (14.0f64).to_bits() & !3);

		let pos_inf = Float::get(Value::from(f64::INFINITY));
		assert!(pos_inf.is_infinite());
		assert!(pos_inf.is_sign_positive());

		let neg_inf = Float::get(Value::from(f64::NEG_INFINITY));
		assert!(neg_inf.is_infinite());
		assert!(neg_inf.is_sign_negative());
	}
}
