use crate::parser::ast::{Compile, Group};
use crate::parser::Parser;
use crate::vm::{block::Builder, SourceLocation};
use crate::{AnyValue, Result};

use crate::value::ty::{Boolean, Float, Integer, Text};
use crate::value::Gc;

fn run_code(code: &str) -> Result<AnyValue> {
	let mut parser = Parser::new(code, None);
	let mut builder = Builder::new(SourceLocation {}, None);
	let scratch = builder.scratch();

	Group::parse_all(&mut parser)
		.expect("bad parse")
		.compile(&mut builder, scratch);

	builder.build().run(Default::default())
}

#[test]
fn divides() {
	let result = run_code(
		r#"
		Integer.zero? = n -> { n == 0 };
		Integer.divides? = (n, l) -> { (l % n).zero?() };
		12.divides?(24).and(!12.divides?(13))
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Boolean>().unwrap(), true);
}

#[test]
fn square_root() {
	let result = run_code(
		r#"
		Integer.'^' = Integer::'**';
		Integer.'√' = n -> { n ^ 0.5 };
		√16
	"#,
	)
	.unwrap();

	// `√16.0` would be `4.0` and `√16` is `4`?
	assert_eq!(result.downcast::<Float>().unwrap(), 4.0);
}

#[test]
fn fib_set_attr() {
	// NOTE: I'm not sure these semantics are what we want, ie setting an attr on the function means
	// the block its in inherits those attrs.
	let result = run_code(
		r#"
		fib = n -> {
			(n <= 1).then(n.return);

			fibb(n - 1) + fibb(n - 2)
		};

		fib.fibb = fib;
		fib(10)
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn fib_set_parent() {
	// NOTE: This won't be necessary later when i get auto inheritance working.
	let result = run_code(
		r#"
		fib = n -> {
			(n <= 1).then(n.return);

			fib(n - 1) + fib(n - 2)
		};

		fib.__parents__ = [:0];
		fib(10)
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn fib_pass_function() {
	let result = run_code(
		r#"
		fib = (n, fn) -> {
			(n <= 1).then(n.return);

			fn(n - 1, fn) + fn(n - 2, fn)
		};

		fib(10, fib)
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
#[ignore] // TODO: remove ignore. doesnt currently work cause blcoks dont inherit from parents.
fn fib_normal() {
	let result = run_code(
		r#"
		fib = n -> {
			(n <= 1).then(n.return);

			fib(n - 1) + fib(n - 2)
		};

		fib(10)
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn modifying_string_literals_isnt_global() {
	let result = run_code(
		r#"
		modify = { "x".concat("y") };

		modify() + modify()
	"#,
	)
	.unwrap();

	assert_eq!(*result.downcast::<Gc<Text>>().unwrap().as_ref().unwrap(), "xyxy");
}

#[test]
fn assign_and_fetch_from_arrays() {
	let result = run_code(
		r#"
		ary = [9, 12, -99];
		ary[1] = 4;
		ary[0] + ary[1]
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 13);
}

#[test]
fn if_and_while() {
	let result = run_code(
		r#"
		i = 0;
		n = 0;
		while({ i < 10 }, {
			if((i % 2) == 0, {
				:2.n = n + i;
			});

			:1.i = i + 1;
		});
		n
	"#,
	)
	.unwrap();

	assert_eq!(result.downcast::<Integer>().unwrap(), 20);
}

#[test]
fn basic_stackframe_continuation() {
	let result = run_code(
		r#"
		recur = acc -> {
			[acc, :0].return();

			recur(acc + "X")
		};

		tmp = recur("X"); q = tmp[0];
		tmp = tmp[1].resume(); q = q + ":" + tmp[0];
		tmp = tmp[1].resume(); q = q + ":" + tmp[0];
		tmp = tmp[1].resume(); q = q + ":" + tmp[0];
		q
	"#,
	)
	.unwrap();

	assert_eq!(
		result
			.downcast::<Gc<Text>>()
			.unwrap()
			.as_ref()
			.unwrap()
			.as_str(),
		"X:XX:XXX:XXXX"
	);
}
