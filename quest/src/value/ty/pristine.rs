use crate::value::{base::Base, Attributed, AttributedMut, Gc, TryAttributed};
use crate::vm::Args;
use crate::{Intern, Result, ToValue, Value};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Pristine(());
}

impl Pristine {
	#[must_use]
	pub fn instance() -> Value {
		static INSTANCE: once_cell::sync::OnceCell<Gc<Pristine>> = once_cell::sync::OnceCell::new();

		INSTANCE
			.get_or_init(|| {
				let mut builder = Base::<Pristine>::builder(7);

				builder
					.set_attr(
						Intern::__get_attr__,
						RustFn_new!("__get_attr__", function funcs::__get_attr__).to_value(),
					)
					.unwrap();

				builder
					.set_attr(
						Intern::__get_unbound_attr__,
						RustFn_new!("__get_unbound_attr__", function funcs::__get_unbound_attr__)
							.to_value(),
					)
					.unwrap();

				builder
					.set_attr(
						Intern::__set_attr__,
						RustFn_new!("__set_attr__", function funcs::__set_attr__).to_value(),
					)
					.unwrap();

				builder
					.set_attr(
						Intern::__del_attr__,
						RustFn_new!("__del_attr__", function funcs::__del_attr__).to_value(),
					)
					.unwrap();

				builder
					.set_attr(
						Intern::__has_attr__,
						RustFn_new!("__has_attr__", function funcs::__has_attr__).to_value(),
					)
					.unwrap();

				builder
					.set_attr(
						Intern::__call_attr__,
						RustFn_new!("__call_attr__", function funcs::__call_attr__).to_value(),
					)
					.unwrap();

				// we don't set parents, as empty parents is default.
				unsafe { builder.finish() }
			})
			.to_value()
	}
}

#[allow(non_snake_case)]
pub mod funcs {
	use super::*;

	pub fn __has_attr__(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(obj.has_attr(args[0])?.to_value())
	}

	pub fn __get_attr__(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		obj.try_get_attr(args[0])
	}

	pub fn __get_unbound_attr__(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		obj.try_get_unbound_attr(args[0])
	}

	pub fn __call_attr__(obj: Value, args: Args<'_>) -> Result<Value> {
		let (attr, args) = args.split_first()?;
		obj.call_attr(attr, args)
	}

	pub fn __set_attr__(mut obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;

		obj.set_attr(args[0], args[1])?;
		Ok(obj)
	}

	pub fn __del_attr__(mut obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(obj.del_attr(args[0])?.unwrap_or_default())
	}
}
