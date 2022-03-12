use crate::value::{Gc, NamedType, AsAny};
use crate::{AnyValue, Result};
use crate::vm::Args;

quest_type! {
	#[derive(Debug)]
	pub struct Pristine(());
}

impl NamedType for Gc<Pristine> {
	const TYPENAME: &'static str = "Pristine";
}

impl Pristine {
	pub fn new() -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		let inner = Base::new_with_parent((), Gc::<Self>::parent());

		unsafe {
			std::mem::transmute(inner)
		}
	}
}

impl Gc<Pristine> {
	#[allow(non_snake_case)]
	pub fn qs__has_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let attr = args.get(0).unwrap();
		Ok(obj.has_attr(attr)?.as_any())
	}

	#[allow(non_snake_case)]
	pub fn qs__get_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let attr = args.get(0).unwrap();
		obj.get_attr(attr)?
			.ok_or_else(|| crate::Error::UnknownAttribute(obj, attr))
	}

	#[allow(non_snake_case)]
	pub fn qs__get_unbound_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let attr = args.get(0).unwrap();
		obj.get_unbound_attr(attr)?
			.ok_or_else(|| crate::Error::UnknownAttribute(obj, attr))
	}

	#[allow(non_snake_case)]
	pub fn qs__call_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let attr = args.get(0)?;

		obj.call_attr(attr, Args::new(&args.positional()[1..], args.keyword()))
	}

	#[allow(non_snake_case)]
	pub fn qs__set_attr__(mut obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;
		let attr = args.get(0).unwrap();
		let value = args.get(1).unwrap();

		obj.set_attr(attr, value)?;
		Ok(obj)
	}

	#[allow(non_snake_case)]
	pub fn qs__del_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let attr = args.get(0).unwrap();
		Ok(obj.del_attr(attr)?.unwrap_or(crate::Value::NULL.any()))
	}
}

quest_type_attrs! { for Gc<Pristine>;
	"__get_attr__" => func qs__get_attr__,
	"__get_bound_attr__" => func qs__get_unbound_attr__,
	"__set_attr__" => func qs__set_attr__,
	"__del_attr__" => func qs__del_attr__,
	"__has_attr__" => func qs__has_attr__,
	"__call_attr__" => func qs__call_attr__,
}
