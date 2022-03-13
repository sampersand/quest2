use crate::value::{Gc, AsAny};
use crate::{AnyValue, Result};
use crate::vm::Args;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Object(());
}

impl Object {
	pub fn instance() -> Gc<Self> {
		static INSTANCE: once_cell::sync::OnceCell<Gc<Object>> = once_cell::sync::OnceCell::new();

		*INSTANCE.get_or_init(|| {
			use crate::value::base::{Base, HasDefaultParent};

			let inner = Base::new_with_parent((), Gc::<Self>::parent());

			unsafe {
				std::mem::transmute(inner)
			}
		})
	}
}

impl Gc<Object> {
	pub fn qs_eql(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((obj.bits() == args[0].bits()).as_any())
	}

	pub fn qs_neq(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		obj.call_attr("==", args)?
			.call_attr("!", Default::default())
	}

	pub fn qs_not(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok((!obj.is_truthy()?).as_any())
	}

	pub fn qs_at_bool(_obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok(true.as_any())
	}

	pub fn qs_at_text(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok(format!("{:?}", obj).as_any())
	}

	pub fn qs_hash(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok((obj.bits() as crate::value::ty::Integer).as_any())
	}

	pub fn qs_clone(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("clone")
	}

	pub fn qs_print(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("print")
	}

	pub fn qs_return(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("return")
	}

	pub fn qs_assert(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("assert")
	}

	pub fn qs_tap(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		Self::qs_tap_into(obj, args).and(Ok(obj))
	}

	pub fn qs_tap_into(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;
		func.call(obj, args)
	}

	pub fn qs_then(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			func.call_no_obj(args)
		} else {
			Ok(obj)
		}
	}

	pub fn qs_and_then(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			func.call(obj, args)
		} else {
			Ok(obj)
		}
	}

	pub fn qs_else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call_no_obj(args)
		}
	}

	pub fn qs_or_else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call(obj, args)
		}
	}

	pub fn qs_or(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			Ok(args[0])
		}
	}

	pub fn qs_and(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy()? {
			Ok(args[0])
		} else {
			Ok(obj)
		}
	}

	pub fn qs_itself(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("itself (probs implemented via bound function)")
	}
}

quest_type_attrs! { for Gc<Object>,
	parent Pristine;
	"==" => func Gc::<Object>::qs_eql,
	"!=" => func Gc::<Object>::qs_neq,
	"!" => func Gc::<Object>::qs_not,

	"@bool" => func Gc::<Object>::qs_at_bool,
	"@text" => func Gc::<Object>::qs_at_text,
	"hash" => func Gc::<Object>::qs_hash,
	"clone" => func Gc::<Object>::qs_clone,

	// "print" => func Gc::<Object>::qs_print,
	// "return" => func Gc::<Object>::qs_return,
	// "assert" => func Gc::<Object>::qs_assert,

	/*
	tap      : a -> (a -> b) -> a
	tap_into : a -> (a -> b) -> b
	then     : a -> (() -> b) -> {a/b}, a if its falsey
	and_then : a -> (a -> b) -> {a/b}, a if its falsey
	else     : a -> (() -> b) -> {a/b}, a if its truthy
	or_else  : a -> (a -> b) -> {a/b}, a if its truthy
	or       : a -> b -> {a/b}, a if its truthy
	and      : a -> b -> {a/b}, a if its falsey
	*/
	"tap" => func Gc::<Object>::qs_tap,
	"tap_into" => func Gc::<Object>::qs_tap_into,
	"then" => func Gc::<Object>::qs_then,
	"and_then" => func Gc::<Object>::qs_and_then,
	"else" => func Gc::<Object>::qs_else,
	"or_else" => func Gc::<Object>::qs_or_else,
	"or" => func Gc::<Object>::qs_or,
	"and" => func Gc::<Object>::qs_and,
	"itself" => func Gc::<Object>::qs_itself,

	// "extend" => func Gc::<Object>::qs_extend,
	// "inherit" => func Gc::<Object>::qs_inherit,
	// "becomes" => func Gc::<Object>::qs_becomes,
}
