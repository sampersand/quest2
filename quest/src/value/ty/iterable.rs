use crate::value::ty::List;
use crate::value::{Callable, ToValue};
use crate::vm::Args;
use crate::{iterator, ErrorKind, Intern, Result, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Iterable;

impl crate::value::NamedType for Iterable {
	const TYPENAME: crate::value::Typename = "Iterable";
}

fn next(iter: Value) -> Result<Value> {
	iter.call_attr(Intern::next, Args::default())
}

macro_rules! for_each {
	($var:pat in $iterable:ident $body:block) => {
		loop {
			match next($iterable) {
				Ok($var) => $body,
				Err(err) if matches!(err.kind, ErrorKind::StopIteration) => break,
				Err(err) => return Err(err),
			}
		}
	};
}

fn reduce_down<F>(iterable: Value, init: Option<Value>, mut func: F) -> Result<Value>
where
	F: FnMut(Value, Value) -> Result<Value>,
{
	let mut current = init.map(Ok).unwrap_or_else(|| next(iterable))?;

	loop {
		match next(iterable) {
			Ok(value) => current = func(value, current)?,
			Err(err) if matches!(err.kind, ErrorKind::StopIteration) => return Ok(current),
			Err(err) => return Err(dbg!(err)),
		}
	}
}

pub mod funcs {
	use super::*;

	pub fn map(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let map_function = args[0];

		Ok(iterator! { "Iterable::max";
			map_function.call(Args::new(&[next(iterable)?], &[]))
		}
		.to_value())
	}

	pub fn filter(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let filter_function = args[0];

		Ok(iterator! { "Iterable::filter";
			loop {
				let ele = next(iterable)?;

				if filter_function.call(Args::new(&[ele], &[]))?.is_truthy() {
					return Ok(ele);
				}
			}
		}
		.to_value())
	}

	pub fn reject(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let filter_function = args[0];

		Ok(iterator! { "Iterable::reject";
			loop {
				let ele = next(iterable)?;

				if !filter_function.call(Args::new(&[ele], &[]))?.is_truthy() {
					return Ok(ele);
				}
			}
		}
		.to_value())
	}

	pub fn reduce(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|x| x.len() == 1 || x.len() == 2)?;

		let (init, func) = match args.len() {
			1 => (None, args[0]),
			2 => (Some(args[0]), args[1]),
			_ => unreachable!(),
		};

		reduce_down(iterable, init, |current, next| func.call(Args::new(&[current, next], &[])))
	}

	pub fn sum(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|x| x.len() <= 1)?;

		reduce_down(iterable, args.get(0), |current, next| {
			current.call_attr(Intern::op_add, Args::new(&[next], &[]))
		})
	}

	pub fn product(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|x| x.len() <= 1)?;

		reduce_down(iterable, args.get(0), |current, next| {
			current.call_attr(Intern::op_mul, Args::new(&[next], &[]))
		})
	}

	pub fn to_list(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		let mut list = List::simple_builder();

		for_each!(value in iterable {
			list.push(value);
		});

		Ok(list.to_value())
	}

	pub fn each(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let func = args[0];

		for_each!(value in iterable {
			func.call(Args::new(&[value], &[]))?;
		});

		Ok(Value::default())
	}

	pub fn tap_each(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let func = args[0];

		Ok(iterator! { "Iterable::tap_each";
			let value = next(iterable)?;
			func.call(Args::new(&[value], &[]))?;
			Ok(value)
		}
		.to_value())
	}

	pub fn count(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		let mut count = 0;

		for_each!(_ in iterable {
			count += 1
		});

		Ok(count.to_value())
	}

	pub fn any(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let condition = args[0];

		for_each!(value in iterable {
			if condition.call(Args::new(&[value], &[]))?.is_truthy() {
				return Ok(true.to_value())
			}
		});

		Ok(false.to_value())
	}

	pub fn all(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let condition = args[0];

		for_each!(value in iterable {
			if !condition.call(Args::new(&[value], &[]))?.is_truthy() {
				return Ok(false.to_value())
			}
		});

		Ok(true.to_value())
	}

	pub fn includes(iterable: Value, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let arg = args[0];

		for_each!(value in iterable {
			if value.try_eq(arg)? {
				return Ok(true.to_value())
			}
		});

		Ok(false.to_value())
	}
}

impl Iterable {
	#[must_use]
	pub fn instance() -> Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Iterable", parent Object::instance();
				Intern::map => function funcs::map,
				Intern::filter => function funcs::filter,
				Intern::reject => function funcs::reject,
				Intern::reduce => function funcs::reduce,
				Intern::each => function funcs::each,
				Intern::to_list => function funcs::to_list,
				Intern::tap_each => function funcs::tap_each,
				Intern::product => function funcs::product,
				Intern::sum => function funcs::sum,
				Intern::count => function funcs::count,
				Intern::any => function funcs::any,
				Intern::all => function funcs::all,
				Intern::includes => function funcs::includes,
			}
		})
	}
}
