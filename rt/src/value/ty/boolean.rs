use crate::value::base::{HasParents, Parents};
use crate::value::ty::{ConvertTo, Integer, Text};
use crate::value::{AnyValue, Convertible, Gc, Value};
use crate::vm::Args;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Boolean(pub bool);

impl super::AttrConversionDefined for Boolean {
	const ATTR_NAME: &'static str = "@bool";
}

impl Value<Boolean> {
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b001_100) };
	pub const TRUE: Self = unsafe { Self::from_bits_unchecked(0b010_100) };
}

impl From<Boolean> for Value<Boolean> {
	fn from(boolean: Boolean) -> Self {
		if boolean.0 {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

impl From<bool> for Value<Boolean> {
	fn from(b: bool) -> Self {
		Boolean(b).into()
	}
}

impl From<Boolean> for bool {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		boolean.0
	}
}

impl From<bool> for Boolean {
	#[inline]
	fn from(b: bool) -> Self {
		Self(b)
	}
}

impl PartialEq<bool> for Boolean {
	fn eq(&self, rhs: &bool) -> bool {
		self.0 == *rhs
	}
}

unsafe impl Convertible for Boolean {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::TRUE.bits() || value.bits() == Value::FALSE.bits()
	}

	fn get(value: Value<Self>) -> Self::Output {
		Self(value.bits() == Value::TRUE.bits())
	}
}

impl HasParents for Boolean {
	unsafe fn init() {
		// todo
	}

	fn parents() -> Parents {
		Default::default() // todo
	}
}

impl ConvertTo<Gc<Text>> for Boolean {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_arguments()?;

		Ok(Text::from_static_str(if self.0 { "true" } else { "false" }))
	}
}

impl ConvertTo<Integer> for Boolean {
	fn convert(&self, args: Args<'_>) -> Result<Integer> {
		args.assert_no_arguments()?;

		Ok(Integer(if self.0 { 1 } else { 0 }))
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

	#[test]
	fn convert_to_text() {
		assert_eq!(
			"true",
			ConvertTo::<Gc<Text>>::convert(&Boolean(true), Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);

		assert_eq!(
			"false",
			ConvertTo::<Gc<Text>>::convert(&Boolean(false), Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
		assert!(
			ConvertTo::<Gc<Text>>::convert(&Boolean(true), Args::new(&[Value::TRUE.any()], &[]))
				.is_err()
		);
	}

	#[test]
	fn convert_to_integer() {
		assert_eq!(
			Integer(1),
			ConvertTo::<Integer>::convert(&Boolean(true), Args::default()).unwrap()
		);

		assert_eq!(
			Integer(0),
			ConvertTo::<Integer>::convert(&Boolean(false), Args::default()).unwrap()
		);
		assert!(
			ConvertTo::<Integer>::convert(&Boolean(true), Args::new(&[Value::TRUE.any()], &[]))
				.is_err()
		);
	}
}
