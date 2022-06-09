use crate::value::{Intern, ToValue};
use crate::vm::Args;
use crate::{Error, ErrorKind, Result, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Object;

impl crate::value::NamedType for Object {
	const TYPENAME: crate::value::Typename = "Object";
}

impl Object {
	#[must_use]
	pub fn instance() -> Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Object", parent Pristine::instance();
				Intern::op_eql => function funcs::eql,
				Intern::op_neq => function funcs::neq,
				Intern::op_not => function funcs::not,
				Intern::to_bool => function funcs::to_bool,
				Intern::to_text => function funcs::to_text,
				Intern::hash => function funcs::hash,
				Intern::r#return => function funcs::r#return,
				Intern::tap => function funcs::tap,
				Intern::then => function funcs::then,
				Intern::r#else => function funcs::r#else,
				Intern::or => function funcs::or,
				Intern::and => function funcs::and,
				Intern::itself => function funcs::itself,
				Intern::display => function funcs::display,
				Intern::freeze => function funcs::freeze,
				Intern::dbg => function funcs::dbg,
				Intern::assert => function funcs::assert,
			}
		})
	}
}

pub mod funcs {
	use super::*;

	pub fn eql(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((obj.id() == args[0].id()).to_value())
	}

	pub fn neq(obj: Value, args: Args<'_>) -> Result<Value> {
		obj.call_attr(Intern::op_eql, args)?.call_attr(Intern::op_not, Args::default())
	}

	pub fn not(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((!obj.is_truthy()).to_value())
	}

	pub fn to_bool(_obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(true.to_value())
	}

	pub fn to_text(obj: Value, args: Args<'_>) -> Result<Value> {
		obj.call_attr(Intern::dbg, args)
	}

	pub fn hash(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((obj.id() as i64).to_value())
	}

	pub fn clone(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		let _ = obj;
		todo!("clone")
	}

	pub fn r#return(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() <= 1)?;

		Err(Error {
			kind: ErrorKind::Return { value: obj, from_frame: args.get(0) },
			stacktrace: crate::error::Stacktrace::empty(),
		})
	}

	pub fn tap(obj: Value, args: Args<'_>) -> Result<Value> {
		tap_into(obj, args).and(Ok(obj))
	}

	pub fn tap_into(obj: Value, args: Args<'_>) -> Result<Value> {
		let (func, args) = args.split_first()?;
		func.call(args.with_this(obj))
	}

	pub fn then(obj: Value, args: Args<'_>) -> Result<Value> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy() {
			func.call(args)
		} else {
			Ok(obj)
		}
	}

	pub fn and_then(obj: Value, args: Args<'_>) -> Result<Value> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy() {
			func.call(args.with_this(obj))
		} else {
			Ok(obj)
		}
	}

	pub fn r#else(obj: Value, args: Args<'_>) -> Result<Value> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy() {
			Ok(obj)
		} else {
			func.call(args)
		}
	}

	pub fn or_else(obj: Value, args: Args<'_>) -> Result<Value> {
		let (func, args) = args.split_first()?;

		if obj.is_truthy() {
			Ok(obj)
		} else {
			func.call(args.with_this(obj))
		}
	}

	pub fn or(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy() {
			Ok(obj)
		} else {
			Ok(args[0])
		}
	}

	pub fn and(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if obj.is_truthy() {
			Ok(args[0])
		} else {
			Ok(obj)
		}
	}

	pub fn itself(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(obj)
	}

	pub fn display(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		crate::value::ty::kernel::funcs::print(Args::new(&[obj], &[])).and(Ok(obj))
	}

	pub fn freeze(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		obj.freeze()?;

		Ok(obj)
	}

	pub fn dbg(obj: Value, args: Args<'_>) -> Result<Value> {
		use crate::value::ty::text::SimpleBuilder;

		args.assert_no_arguments()?;
		let typename = obj.typename();

		let mut builder = SimpleBuilder::with_capacity(21 + typename.len());
		builder.push('<');
		builder.push_str(typename);
		builder.push(':');
		builder.push_str(&format!("{:p}", obj.bits() as *const u8));
		builder.push('>');

		Ok(builder.finish().to_value())
	}

	pub fn assert(obj: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() <= 1)?;

		if obj.is_truthy() {
			Ok(obj)
		} else {
			Err(ErrorKind::AssertionFailed(args.get(0).map(Value::try_downcast).transpose()?).into())
		}
	}
}
/*
singleton_object! { for Object, parent Pristine;
	"==" => func!(funcs::eql),
	"!=" => func!(funcs::neq),
	"!" => func!(funcs::not),

	"@bool" => func!(funcs::to_bool),
	"@text" => func!(funcs::to_text),
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
