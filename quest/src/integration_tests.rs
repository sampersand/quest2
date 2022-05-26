use crate::parse::ast::{Compile, Group};
use crate::parse::Parser;
use crate::vm::{
	block::{Builder, Local},
	Args,
};
use crate::{AnyValue, Result};

use crate::value::ty::{Boolean, Float, Integer, List, Text};
use crate::value::Gc;

pub fn run_code(code: &str) -> Result<AnyValue> {
	let mut parser = Parser::new(code, None);
	let mut builder = Builder::default();

	Group::parse_all(&mut parser)
		.expect("bad parse")
		.compile(&mut builder, Local::Scratch);

	builder.build().run(Args::default())
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
#[ignore] // This is currently a bug
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

#[test]
fn dbg_representations() {
	let result = run_code(
		r#"
			block = { :0 };
			frame = block();
			[true, false, null, 12."+", 1.12, 1, "f\n", frame, block].dbg()
		"#,
	)
	.unwrap();

	// We don't actually check the return value as it's not defined exactly.
	assert!(result.is_a::<Gc<Text>>());

	// Also note we don't test `Integer` and friends debug representations. See issue #23
}

#[test]
fn basic_syntax() {
	let result = run_code(
		r#"
			$syntax { 12 $bar:(3 $| 4) } = { 12 - $bar };
			12 3
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), 9);

	let result = run_code(
		r#"
			$syntax { 12 $bar:(3 $| 4) } = { 12 - $bar };
			12 4
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), 8);
}

#[test]
fn nested_syntax() {
	let result = run_code(
		r#"
			$syntax { defn $name:(a $| b) } = {
				$$syntax { $name } = { 3 - };
			};

			defn a
			(a 10) * (a 0)
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), -21);
}

#[test]
fn if_while_and_do_while() {
	let result = run_code(
		r#"
			$syntax { if $cond:group $body:block } = { (if)($cond, $body); };
			$syntax { while $cond:group $body:block } = { (while)({ $cond }, $body); };
			$syntax { do $body:block while $cond:group } = { $body(); while $cond $body };

			x = 0;
			i = 0;
			do {
				:-1.x = x + 1;
				if (0 != x % 2) {
					:-1.i = i + x;
				}
			} while (x < 10);
			i
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), 25);
}

#[test]
fn alias_macro() {
	let result = run_code(
		r#"
			$syntax { alias $new:token $orig:token ; } = {
			  $$syntax { $new } = { $orig };
			};

			alias <- = ;
			alias __current_stackframe__ :0 ;
			x <- 3;
			__current_stackframe__.x
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), 3);
}

#[test]
fn list_comprehension() {
	let result = run_code(
		r#"
			$syntax {
				[ $body:tt | $var:ident in $src:tt ]
			} = {
				$src.map($var -> { $body })
			};

			[(x * 2) | x in [1,2,3,4]]
		"#,
	)
	.unwrap();

	let list = result.downcast::<Gc<List>>().unwrap().as_ref().unwrap();
	assert_eq!(list.len(), 4);
	assert_eq!(list.as_slice()[0].downcast::<Integer>().unwrap(), 2);
	assert_eq!(list.as_slice()[1].downcast::<Integer>().unwrap(), 4);
	assert_eq!(list.as_slice()[2].downcast::<Integer>().unwrap(), 6);
	assert_eq!(list.as_slice()[3].downcast::<Integer>().unwrap(), 8);
}

#[test]
fn lists_containing_themselves() {
	let result = run_code(
		r#"
			l = [1,2];
			l[0] = [3,4];
			l[0][0] = l[0];
			l[1] = l[0];
			l.dbg()
		"#,
	)
	.unwrap();

	let result = result.downcast::<Gc<Text>>().unwrap().as_ref().unwrap();
	assert_eq!(result.as_str(), "[[[...], 4], [[...], 4]]");
}

#[test]
fn reference_syntax_groups() {
	let result = run_code(
		r#"
			$syntax time { $hr:int : $min:int } = { ($min + $hr*60) } ;
			$syntax { $t:time am } = { $t } ;
			$syntax { $t:time pm } = { ($t + 3600) } ;

			(10 : 30 am) * (10 : 30 pm)
		"#,
	)
	.unwrap();
	assert_eq!(result.downcast::<Integer>().unwrap(), 630*4230);
}
