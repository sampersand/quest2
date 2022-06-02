use crate::value::ty::{ConvertTo, InstanceOf, Singleton, Text};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FloatClass;

impl Singleton for FloatClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Float", parent Object::instance();
				// Intern::op_add => method funcs::add,
				// Intern::op_sub => method funcs::sub,
				// Intern::op_mul => method funcs::mul,
				// Intern::op_div => method funcs::div,
				// Intern::op_mod => method funcs::r#mod,
				// Intern::op_pow => method funcs::pow,
				// Intern::op_lth => method funcs::lth,
				// Intern::op_leq => method funcs::leq,
				// Intern::op_neg => method funcs::neg,
				Intern::at_text => method funcs::at_text,
				Intern::dbg => method funcs::dbg,
			}
		})
	}
}

impl InstanceOf for Float {
	type Parent = FloatClass;
}

pub mod funcs {
	use super::*;

	pub fn at_text(float: Float, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Gc<Text>>::convert(&float, args).map(ToValue::to_value)
	}

	pub fn dbg(float: Float, args: Args<'_>) -> Result<Value> {
		at_text(float, args)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::ToValue;

	#[test]
	fn test_is_a() {
		assert!(Float::is_a(Value::from(0.0).to_value()));
		assert!(Float::is_a(Value::from(1.0).to_value()));
		assert!(Float::is_a(Value::from(-123.456).to_value()));
		assert!(Float::is_a(Value::from(14.0).to_value()));
		assert!(Float::is_a(Value::from(f64::NAN).to_value()));
		assert!(Float::is_a(Value::from(f64::INFINITY).to_value()));
		assert!(Float::is_a(Value::from(f64::NEG_INFINITY).to_value()));

		assert!(!Float::is_a(Value::TRUE.to_value()));
		assert!(!Float::is_a(Value::FALSE.to_value()));
		assert!(!Boolean::is_a(Value::NULL.to_value()));
		assert!(!Float::is_a(Value::ZERO.to_value()));
		assert!(!Float::is_a(Value::ONE.to_value()));
		assert!(!Float::is_a(Value::from("hello").to_value()));
		assert!(!Float::is_a(Value::from(RustFn::NOOP).to_value()));
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
