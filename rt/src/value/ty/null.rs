use crate::value::ty::{Boolean, ConvertTo, Float, Integer, List, Text};
use crate::value::{AsAny, AnyValue, Convertible, Gc, Value};
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
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b1000) };
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

impl Null {
	pub fn qs_at_text(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Gc<Text>>::convert(&self, args).map(AsAny::as_any)
	}

	pub fn qs_at_int(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Integer>::convert(&self, args).map(AsAny::as_any)
	}

	pub fn qs_at_float(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Float>::convert(&self, args).map(AsAny::as_any)
	}

	pub fn qs_at_list(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Gc<List>>::convert(&self, args).map(AsAny::as_any)
	}

	pub fn qs_at_bool(self, args: Args<'_>) -> Result<AnyValue> {
		ConvertTo::<Boolean>::convert(&self, args).map(AsAny::as_any)
	}

	pub fn qs_inspect(self, args: Args<'_>) -> Result<AnyValue> {
		self.qs_at_text(args)
	}
}

quest_type_attrs! { for Null;
	"inspect" => qs_inspect,
	"@text" => qs_at_text,
	"@int" => qs_at_int,
	"@float" => qs_at_float,
	"@list" => qs_at_list,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;

	#[test]
	fn test_is_a() {
		assert!(Null::is_a(Default::default()));

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
