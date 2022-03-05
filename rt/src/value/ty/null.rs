use crate::value::base::{HasParents, Parents};
use crate::value::ty::{Boolean, ConvertTo, Integer, List, Text, Float};
use crate::value::{AnyValue, Convertible, Gc, Value};
use crate::vm::Args;
use crate::Result;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "null")
	}
}

impl Value<Null> {
	pub const NULL: Self = unsafe { Self::from_bits_unchecked(0b011_100) };
}

impl From<Null> for Value<Null> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

unsafe impl Convertible for Null {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() == Value::NULL.bits()
	}

	fn get(_: Value<Self>) -> Self::Output {
		Self
	}
}

// static mut NULL_PARENT: MaybeUninit<Base<Scope>> = MaybeUninit::uninit();

impl HasParents for Null {
	unsafe fn init() {
		// let builder = Base::static_builder(&mut NULL_PARENT);
		// // update the builder
		// builder.finish();
	}

	fn parents() -> Parents {
		todo!()
		// Parents::new_singular(unsafe { Gc::new_unchecked(std::ptr::addr_of_mut!(NULL_PARENT).cast::<Base<Scope>>()) })
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

		Ok(Integer(0))
	}
}

impl ConvertTo<Float> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		Ok(Float(0.0))
	}
}

impl ConvertTo<Boolean> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Boolean> {
		args.assert_no_arguments()?;

		Ok(Boolean(false))
	}
}

impl ConvertTo<Gc<List>> for Null {
	fn convert(&self, args: Args<'_>) -> Result<Gc<List>> {
		args.assert_no_arguments()?;

		Ok(List::new())
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
	fn conversions() {
		assert_eq!(
			"null",
			ConvertTo::<Gc<Text>>::convert(&Null, Args::default())
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
		assert!(ConvertTo::<Gc<Text>>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());

		assert_eq!(
			Integer(0),
			ConvertTo::<Integer>::convert(&Null, Args::default()).unwrap()
		);
		assert!(ConvertTo::<Integer>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());

		assert_eq!(
			Boolean(false),
			ConvertTo::<Boolean>::convert(&Null, Args::default()).unwrap()
		);
		assert!(ConvertTo::<Boolean>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());

		assert!(ConvertTo::<Gc<List>>::convert(&Null, Args::default())
			.unwrap()
			.as_ref()
			.unwrap()
			.is_empty());
		assert!(ConvertTo::<Gc<List>>::convert(&Null, Args::new(&[Value::TRUE.any()], &[])).is_err());
	}
}
