use crate::value::ty::{Boolean, ConvertTo, Float, InstanceOf, Integer, List, Singleton, Text};
use crate::value::{AnyValue, Convertible, Gc, ToAny, Value};
use crate::vm::Args;
use crate::Result;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl crate::value::NamedType for Null {
	const TYPENAME: &'static str = "Null";
}

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "null")
	}
}

impl Value<Null> {
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b0001_0100) };
}

impl From<Null> for Value<Null> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl Convertible for Null {
	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::NULL.bits()
	}

	fn get(_: Value<Self>) -> Self {
		Self
	}
}

impl ConvertTo<Gc<Text>> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_arguments()?;

		Ok(Text::from_static_str("null"))
	}
}

impl ConvertTo<Integer> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Integer> {
		args.assert_no_arguments()?;

		Ok(0)
	}
}

impl ConvertTo<Float> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		Ok(0.0)
	}
}

impl ConvertTo<Boolean> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Boolean> {
		args.assert_no_arguments()?;

		Ok(false)
	}
}

impl ConvertTo<Gc<List>> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Gc<List>> {
		args.assert_no_arguments()?;

		Ok(List::new())
	}
}

pub mod funcs {
	use super::*;

	pub fn at_text(null: Null, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Gc<Text>>::convert(&null, args).map(ToAny::to_any)
	}

	pub fn at_int(null: Null, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Integer>::convert(&null, args).map(ToAny::to_any)
	}

	pub fn at_float(null: Null, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Float>::convert(&null, args).map(ToAny::to_any)
	}

	pub fn at_list(null: Null, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Gc<List>>::convert(&null, args).map(ToAny::to_any)
	}

	pub fn at_bool(null: Null, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Boolean>::convert(&null, args).map(ToAny::to_any)
	}

	pub fn dbg(null: Null, args: Args<'_>) -> Result<AnyValue> {
		at_text(null, args)
	}
}

impl InstanceOf for Null {
	type Parent = NullClass;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NullClass;

impl Singleton for NullClass {
	fn instance() -> crate::AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Null", parent Object::instance();
				Intern::dbg => method funcs::dbg,
				Intern::at_text => method funcs::at_text,
				Intern::at_int => method funcs::at_int,
				Intern::at_float => method funcs::at_float,
				Intern::at_bool => method funcs::at_bool,
				Intern::at_list => method funcs::at_list,
			}
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;

	#[test]
	fn test_is_a() {
		assert!(Null::is_a(Value::NULL.any()));

		assert!(!Null::is_a(Value::TRUE.any()));
		assert!(!Null::is_a(Value::FALSE.any()));
		assert!(!Null::is_a(Value::ZERO.any()));
		assert!(!Null::is_a(Value::ONE.any()));
		assert!(!Null::is_a(Value::from(1.0).any()));
		assert!(!Null::is_a(Value::from("hello").any()));
		assert!(!Null::is_a(Value::from(RustFn::NOOP).any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Null, Null::get(Value::from(Null)));
	}

	#[test]
	fn test_convert_to_text() {
		assert_eq!(
			"null",
			ConvertTo::<Gc<Text>>::convert(&Null, Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
		assert!(ConvertTo::<Gc<Text>>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}

	#[test]
	fn test_convert_to_integer() {
		assert_eq!(0, ConvertTo::<Integer>::convert(&Null, Args::default()).unwrap());
		assert!(ConvertTo::<Integer>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}

	#[test]
	fn test_convert_to_float() {
		assert_eq!(false, ConvertTo::<Boolean>::convert(&Null, Args::default()).unwrap());
		assert!(ConvertTo::<Boolean>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}

	#[test]
	fn test_convert_to_list() {
		assert!(ConvertTo::<Gc<List>>::convert(&Null, Args::default())
			.unwrap()
			.as_ref()
			.unwrap()
			.is_empty());
		assert!(ConvertTo::<Gc<List>>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}
}
