use crate::value::base::HasDefaultParent;
use crate::value::ty::{ConvertTo, Float, Integer, Text};
use crate::value::{AnyValue, Convertible, Gc, Value};
use crate::vm::Args;
use crate::Result;

pub type Boolean = bool;

impl super::AttrConversionDefined for Boolean {
	const ATTR_NAME: &'static str = "@bool";
}

impl Value<Boolean> {
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b0100) };
	pub const TRUE: Self = unsafe { Self::from_bits_unchecked(0b1100) };
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
	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::TRUE.bits() || value.bits() == Value::FALSE.bits()
	}

	fn get(value: Value<Self>) -> Self {
		value.bits() == Value::TRUE.bits()
	}
}

impl HasDefaultParent for Boolean {
	fn parent() -> crate::AnyValue {
		Default::default()
	}
}

impl ConvertTo<Gc<Text>> for Boolean {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_arguments()?;

		Ok(Text::from_static_str(if *self { "true" } else { "false" }))
	}
}

impl ConvertTo<Integer> for Boolean {
	fn convert(&self, args: Args<'_>) -> Result<Integer> {
		args.assert_no_arguments()?;

		Ok(if *self { 1 } else { 0 })
	}
}

impl ConvertTo<Float> for Boolean {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		Ok(if *self { 1.0 } else { 0.0 })
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

		assert!(!Boolean::is_a(Default::default()));
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

	#[test]
	fn test_convert_to_text() {
		assert_eq!(
			"true",
			ConvertTo::<Gc<Text>>::convert(&true, Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
		assert_eq!(
			"false",
			ConvertTo::<Gc<Text>>::convert(&false, Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);

		assert!(ConvertTo::<Gc<Text>>::convert(&true, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}

	#[test]
	fn test_convert_to_integer() {
		assert_eq!(1, ConvertTo::<Integer>::convert(&true, Args::default()).unwrap());
		assert_eq!(0, ConvertTo::<Integer>::convert(&false, Args::default()).unwrap());

		assert!(ConvertTo::<Integer>::convert(&true, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}

	#[test]
	fn test_convert_to_float() {
		assert_eq!(1.0, ConvertTo::<Float>::convert(&true, Args::default()).unwrap());
		assert_eq!(0.0, ConvertTo::<Float>::convert(&false, Args::default()).unwrap());

		assert!(ConvertTo::<Float>::convert(&true, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}
}
