use crate::value::{Gc, /*AsAny*/};
use crate::{AnyValue, Result};
use crate::vm::Args;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Kernel(());
}

impl Kernel {
	pub fn instance() -> Gc<Self> {
		static INSTANCE: once_cell::sync::OnceCell<Gc<Kernel>> = once_cell::sync::OnceCell::new();

		*INSTANCE.get_or_init(|| {
			use crate::value::base::{Base, HasDefaultParent};

			let inner = Base::new_with_parent((), Gc::<Self>::parent());

			unsafe {
				std::mem::transmute(inner)
			}
		})
	}
}

impl Gc<Kernel> {
	// pub fn qs__has_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	// 	args.assert_no_keyword()?;
	// 	args.assert_positional_len(1)?;

	// 	Ok(obj.has_attr(args[0])?.as_any())
	// }

	// pub fn qs__get_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	// 	args.assert_no_keyword()?;
	// 	args.assert_positional_len(1)?;

	// 	obj.get_attr(args[0])?
	// 		.ok_or_else(|| crate::Error::UnknownAttribute(obj, args[0]))
	// }

	// pub fn qs__get_unbound_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	// 	args.assert_no_keyword()?;
	// 	args.assert_positional_len(1)?;

	// 	obj.get_unbound_attr(args[0])?
	// 		.ok_or_else(|| crate::Error::UnknownAttribute(obj, args[0]))
	// }

	pub fn qs_if(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() == 2 || a.positional().len() == 3)?;

		if args[0].is_truthy()? {
			args[1].call_no_obj(Args::default())
		} else if let Ok(if_false) = args.get(2) {
			if_false.call_no_obj(Args::default())
		} else {
			Ok(crate::Value::NULL.any())
		}
	}

	pub fn qs_while(self, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;

		let mut last = crate::Value::NULL.any();

		while args[0].is_truthy()? {
			last = args[1].call_no_obj(Default::default())?;
		}

		Ok(last)
	}

	// pub fn qs__set_attr__(mut obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	// 	args.assert_no_keyword()?;
	// 	args.assert_positional_len(2)?;

	// 	obj.set_attr(args[0], args[1])?;
	// 	Ok(obj)
	// }

	// pub fn qs__del_attr__(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	// 	args.assert_no_keyword()?;
	// 	args.assert_positional_len(1)?;

	// 	Ok(obj.del_attr(args[0])?.unwrap_or(crate::Value::NULL.any()))
	// }
}

quest_type_attrs! { for Gc<Kernel>, parent Pristine;
	"if" => meth qs_if,
	"while" => meth qs_while,
	// "__get_attr__" => func qs__get_attr__,
	// "__get_bound_attr__" => func qs__get_unbound_attr__,
	// "__set_attr__" => func qs__set_attr__,
	// "__del_attr__" => func qs__del_attr__,
	// "__has_attr__" => func qs__has_attr__,
	// "__call_attr__" => func qs__call_attr__,
}
