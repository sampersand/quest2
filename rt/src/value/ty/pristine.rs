use crate::value::{Gc, AsAny};
use crate::{AnyValue, Result};
use crate::vm::Args;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Pristine(());
}

impl Pristine {
	pub fn instance() -> Gc<Self> {
		static INSTANCE: once_cell::sync::OnceCell<Gc<Pristine>> = once_cell::sync::OnceCell::new();

		*INSTANCE.get_or_init(|| {
			let inner = crate::value::base::Builder::<()>::allocate_with_capacity(6);

			// we don't set parents, as empty parents is default.
			unsafe {
				std::mem::transmute(inner.finish())
			}
		})
	}
}

impl Gc<Pristine> {
	#[allow(non_snake_case)]
	pub fn qs__has_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(obj.has_attr(args[0])?.as_any())
	}

	#[allow(non_snake_case)]
	pub fn qs__get_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		obj.get_attr(args[0])?
			.ok_or_else(|| crate::Error::UnknownAttribute(obj, args[0]))
	}

	#[allow(non_snake_case)]
	pub fn qs__get_unbound_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		obj.get_unbound_attr(args[0])?
			.ok_or_else(|| crate::Error::UnknownAttribute(obj, args[0]))
	}

	#[allow(non_snake_case)]
	pub fn qs__call_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (attr, args) = args.split_first()?;
		obj.call_attr(attr, args)
	}

	#[allow(non_snake_case)]
	pub fn qs__set_attr__(mut obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;

		obj.set_attr(args[0], args[1])?;
		Ok(obj)
	}

	#[allow(non_snake_case)]
	pub fn qs__del_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(obj.del_attr(args[0])?.unwrap_or(crate::Value::NULL.any()))
	}
}

quest_type_attrs! { for Gc<Pristine>;
	"__get_attr__" => func Gc::<Pristine>::qs__get_attr__,
	"__get_bound_attr__" => func Gc::<Pristine>::qs__get_unbound_attr__,
	"__set_attr__" => func Gc::<Pristine>::qs__set_attr__,
	"__del_attr__" => func Gc::<Pristine>::qs__del_attr__,
	"__has_attr__" => func Gc::<Pristine>::qs__has_attr__,
	"__call_attr__" => func Gc::<Pristine>::qs__call_attr__,
}
