use crate::value::ty::{ConvertTo, Float, Integer, Text};
use crate::value::{Convertible, Gc, HasDefaultParent};
use crate::vm::Args;
use crate::{Result, Value};

pub type Boolean = bool;

impl super::AttrConversionDefined for Boolean {
	const ATTR_NAME: crate::value::Intern = crate::value::Intern::at_bool;
}

impl crate::value::NamedType for Boolean {
	const TYPENAME: crate::value::Typename = "Boolean";
}

impl Value<Boolean> {
	pub const FALSE: Self = unsafe { Self::from_bits_unchecked(0b0000_0100) };
	pub const TRUE: Self = unsafe { Self::from_bits_unchecked(0b0010_0100) };
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
	fn is_a(value: Value) -> bool {
		value.bits() == Value::TRUE.bits() || value.bits() == Value::FALSE.bits()
	}

	fn get(value: Value<Self>) -> Self {
		value.bits() == Value::TRUE.bits()
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

pub mod funcs {
	use super::*;

	use crate::value::ToValue;

	pub fn then(boolean: bool, args: Args<'_>) -> Result<Value> {
		if !boolean {
			return Ok(boolean.to_value());
		}

		let (func, args) = args.split_first()?;
		func.call(args)
	}

	pub fn and_then(boolean: bool, args: Args<'_>) -> Result<Value> {
		if !boolean {
			return Ok(boolean.to_value());
		}

		let (func, args) = args.split_first()?;
		func.call(args.with_this(boolean.to_value()))
	}

	pub fn r#else(boolean: bool, args: Args<'_>) -> Result<Value> {
		if boolean {
			return Ok(boolean.to_value());
		}

		let (func, args) = args.split_first()?;
		func.call(args)
	}

	pub fn or_else(boolean: bool, args: Args<'_>) -> Result<Value> {
		if boolean {
			return Ok(boolean.to_value());
		}

		let (func, args) = args.split_first()?;
		func.call(args.with_this(boolean.to_value()))
	}

	pub fn or(boolean: bool, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if boolean {
			return Ok(boolean.to_value());
		}

		Ok(args[0])
	}

	pub fn and(boolean: bool, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if !boolean {
			return Ok(boolean.to_value());
		}

		Ok(args[0])
	}

	pub fn at_text(boolean: bool, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;

		ConvertTo::<Gc<Text>>::convert(&boolean, args).map(ToValue::to_value)
	}

	pub fn dbg(boolean: bool, args: Args<'_>) -> Result<Value> {
		at_text(boolean, args)
	}
}

impl HasDefaultParent for Boolean {
	fn parent() -> Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Boolean", parent Object::instance();
				// "+" => method funcs::add,
				Intern::then => method funcs::then,
				Intern::and_then => method funcs::and_then,
				Intern::r#else => method funcs::r#else,
				Intern::or_else => method funcs::or_else,
				Intern::and => method funcs::and,
				Intern::or => method funcs::or,
				Intern::dbg => method funcs::dbg,
				Intern::at_text => method funcs::at_text,
			}
		})
	}
}

// quest_type! {
// 	#[derive(Debug, NamedType)]
// 	pub struct BooleanClass(());
// }

// singleton_object! { for BooleanClass;
// 	"@text"
// }

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
	fn test_convert_to_text() {
		assert_eq!(
			"true",
			ConvertTo::<Gc<Text>>::convert(&true, Args::default()).unwrap().as_ref().unwrap().as_str()
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
