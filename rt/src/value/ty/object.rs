use crate::value::{AsAny, Intern};
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

		let mut thing = *INSTANCE.get_or_init(|| { first = true; new_quest_scope! { @parent Pristine; }.unwrap().as_any() });

		if first {
			thing.set_attr(Intern::op_eql, RustFn_new!(Intern::op_eql, function funcs::eql).as_any()).unwrap();
			thing.set_attr(Intern::op_neq, RustFn_new!(Intern::op_neq, function funcs::neq).as_any()).unwrap();
			thing.set_attr(Intern::op_not, RustFn_new!(Intern::op_not, function funcs::not).as_any()).unwrap();

			thing.set_attr(Intern::at_bool, RustFn_new!(Intern::at_bool, function funcs::at_bool).as_any()).unwrap();
			thing.set_attr(Intern::at_text, RustFn_new!(Intern::at_text, function funcs::at_text).as_any()).unwrap();
			thing.set_attr(Intern::hash, RustFn_new!(Intern::hash, function funcs::hash).as_any()).unwrap();
			thing.set_attr(Intern::clone, RustFn_new!(Intern::clone, function funcs::clone).as_any()).unwrap();

			thing.set_attr(Intern::tap, RustFn_new!(Intern::tap, function funcs::tap).as_any()).unwrap();
			thing.set_attr(Intern::tap_into, RustFn_new!(Intern::tap_into, function funcs::tap_into).as_any()).unwrap();
			thing.set_attr(Intern::then, RustFn_new!(Intern::then, function funcs::then).as_any()).unwrap();
			thing.set_attr(Intern::and_then, RustFn_new!(Intern::and_then, function funcs::and_then).as_any()).unwrap();
			thing.set_attr(Intern::r#else, RustFn_new!(Intern::r#else, function funcs::r#else).as_any()).unwrap();
			thing.set_attr(Intern::or_else, RustFn_new!(Intern::or_else, function funcs::or_else).as_any()).unwrap();
			thing.set_attr(Intern::or, RustFn_new!(Intern::or, function funcs::or).as_any()).unwrap();
			thing.set_attr(Intern::and, RustFn_new!(Intern::and, function funcs::and).as_any()).unwrap();
			thing.set_attr(Intern::itself, RustFn_new!(Intern::itself, function funcs::itself).as_any()).unwrap();
		}

		thing
	}
}

pub mod funcs {
	use super::*;

	pub fn eql(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((obj.id() == args[0].id()).as_any())
	}

	pub fn neq(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		obj.call_attr(Intern::op_eql, args)?
			.call_attr(Intern::op_not, Args::default())
	}

	pub fn not(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok((!obj.is_truthy()?).as_any())
	}

	pub fn at_bool(_obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok(true.as_any())
	}

	pub fn at_text(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok(format!("{:?}", obj).as_any())
	}

	pub fn hash(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		Ok((obj.bits() as crate::value::ty::Integer).as_any())
	}

	pub fn clone(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("clone")
	}

	pub fn print(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("print")
	}

	pub fn r#return(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("return")
	}

	pub fn assert(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("assert")
	}

	pub fn tap(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		tap_into(obj, args).and(Ok(obj))
	}

	pub fn tap_into(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;
		func.call(args.with_self(obj))
	}

	pub fn then(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			func.call_no_obj(args)
		} else {
			Ok(obj)
		}
	}

	pub fn and_then(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			func.call(args.with_self(obj))
		} else {
			Ok(obj)
		}
	}

	pub fn r#else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call(args)
		}
	}

	pub fn or_else(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			func.call(args.with_self(obj))
		}
	}

	pub fn or(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy()? {
			Ok(obj)
		} else {
			Ok(args[0])
		}
	}

	pub fn and(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy()? {
			Ok(args[0])
		} else {
			Ok(obj)
		}
	}

	pub fn itself(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("itself (probs implemented via bound function)")
	}
}
/*
singleton_object! { for Object, parent Pristine;
	"==" => func!(funcs::eql),
	"!=" => func!(funcs::neq),
	"!" => func!(funcs::not),

	"@bool" => func!(funcs::at_bool),
	"@text" => func!(funcs::at_text),
	"hash" => func!(funcs::hash),
	"clone" => func!(funcs::clone),

	// "print" => func!(funcs::print),
	// "return" => func!(funcs::return),
	// "assert" => func!(funcs::assert),

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
	"tap" => func!(funcs::tap),
	"tap_into" => func!(funcs::tap_into),
	"then" => func!(funcs::then),
	"and_then" => func!(funcs::and_then),
	"else" => func!(funcs::else),
	"or_else" => func!(funcs::or_else),
	"or" => func!(funcs::or),
	"and" => func!(funcs::and),
	"itself" => func!(funcs::itself),

	// "extend" => func!(funcs::extend),
	// "inherit" => func!(funcs::inherit),
	// "becomes" => func!(funcs::becomes),
}
*/
