use crate::value::{AsAny, Intern};
use crate::vm::Args;
use crate::{AnyValue, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Object;

impl crate::value::NamedType for Object {
	const TYPENAME: &'static str = "Object";
}

impl Object {
	pub fn instance() -> AnyValue {
		use ::once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Object", parent Pristine::instance();
				Intern::op_eql => function funcs::eql,
				Intern::op_neq => function funcs::neq,
				Intern::op_not => function funcs::not,
				Intern::at_bool => function funcs::at_bool,
				Intern::at_text => function funcs::at_text,
				Intern::hash => function funcs::hash,
				Intern::r#return => function funcs::r#return,
				Intern::tap => function funcs::tap,
				Intern::then => function funcs::then,
				Intern::r#else => function funcs::r#else,
				Intern::or => function funcs::or,
				Intern::and => function funcs::and,
				Intern::itself => function funcs::itself,
			}
		})
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
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() <= 1)?;

		let from_frame = if let Ok(index) = args.get(0) {
			index
		} else {
			crate::vm::Frame::with_stackframe(|sfs| *sfs.last().expect("returning from nothing?"))
				.as_any()
		};

		Err(crate::Error::Return {
			value: obj,
			from_frame,
		})
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
			func.call(args)
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