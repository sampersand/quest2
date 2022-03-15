use crate::value::AsAny;
use crate::{AnyValue, Result};
use crate::vm::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Object;

impl crate::value::NamedType for Object {
	const TYPENAME: &'static str = "Object";
}

impl Object {
	pub fn instance() -> AnyValue {

		use ::once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

		let mut first = false;

		let mut thing = *INSTANCE.get_or_init(|| { first = true; new_quest_scope! { parent Pristine; }.unwrap().as_any() });

		if first {
			thing.set_attr("==", RustFn_new!("==", function funcs::qs_eql).as_any()).unwrap();
			thing.set_attr("!=", RustFn_new!("!=", function funcs::qs_neq).as_any()).unwrap();
			thing.set_attr("!", RustFn_new!("!", function funcs::qs_not).as_any()).unwrap();

			thing.set_attr("@bool", RustFn_new!("@bool", function funcs::qs_at_bool).as_any()).unwrap();
			thing.set_attr("@text", RustFn_new!("@text", function funcs::qs_at_text).as_any()).unwrap();
			thing.set_attr("hash", RustFn_new!("hash", function funcs::qs_hash).as_any()).unwrap();
			thing.set_attr("clone", RustFn_new!("clone", function funcs::qs_clone).as_any()).unwrap();

			thing.set_attr("tap", RustFn_new!("tap", function funcs::qs_tap).as_any()).unwrap();
			thing.set_attr("tap_into", RustFn_new!("tap_into", function funcs::qs_tap_into).as_any()).unwrap();
			thing.set_attr("then", RustFn_new!("then", function funcs::qs_then).as_any()).unwrap();
			thing.set_attr("and_then", RustFn_new!("and_then", function funcs::qs_and_then).as_any()).unwrap();
			thing.set_attr("else", RustFn_new!("else", function funcs::qs_else).as_any()).unwrap();
			thing.set_attr("or_else", RustFn_new!("or_else", function funcs::qs_or_else).as_any()).unwrap();
			thing.set_attr("or", RustFn_new!("or", function funcs::qs_or).as_any()).unwrap();
			thing.set_attr("and", RustFn_new!("and", function funcs::qs_and).as_any()).unwrap();
			thing.set_attr("itself", RustFn_new!("itself", function funcs::qs_itself).as_any()).unwrap();
		}

		thing
	}
}

pub mod funcs {
	use super::*;

	pub fn qs_eql(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((obj.id() == args[0].id()).as_any())
	}

	pub fn qs_neq(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		obj.call_attr("==", args)?
			.call_attr("!", Args::default())
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
		qs_tap_into(obj, args).and(Ok(obj))
	}

	pub fn qs_tap_into(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;
		func.call(args.with_self(obj))
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
			func.call(args.with_self(obj))
		} else {
			Ok(obj)
		}
	}

	pub fn qs_else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call(args)
		}
	}

	pub fn qs_or_else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call(args.with_self(obj))
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
/*
singleton_object! { for Object, parent Pristine;
	"==" => func!(funcs::qs_eql),
	"!=" => func!(funcs::qs_neq),
	"!" => func!(funcs::qs_not),

	"@bool" => func!(funcs::qs_at_bool),
	"@text" => func!(funcs::qs_at_text),
	"hash" => func!(funcs::qs_hash),
	"clone" => func!(funcs::qs_clone),

	// "print" => func!(funcs::qs_print),
	// "return" => func!(funcs::qs_return),
	// "assert" => func!(funcs::qs_assert),

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
	"tap" => func!(funcs::qs_tap),
	"tap_into" => func!(funcs::qs_tap_into),
	"then" => func!(funcs::qs_then),
	"and_then" => func!(funcs::qs_and_then),
	"else" => func!(funcs::qs_else),
	"or_else" => func!(funcs::qs_or_else),
	"or" => func!(funcs::qs_or),
	"and" => func!(funcs::qs_and),
	"itself" => func!(funcs::qs_itself),

	// "extend" => func!(funcs::qs_extend),
	// "inherit" => func!(funcs::qs_inherit),
	// "becomes" => func!(funcs::qs_becomes),
}
*/
