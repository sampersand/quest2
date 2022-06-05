use quest::parse::ast::{Compile, Group};
use quest::parse::Parser;
use quest::vm::{
	block::{Builder, Local},
	Args,
};
use quest::{Result, Value};

use quest::value::ty::{Boolean, Float, Integer, List, Text};
use quest::value::Gc;

pub fn run_code(code: &str) -> Result<Value> {
	let mut parser = Parser::new(code, None);
	let mut builder = Builder::default();

	Group::parse_all(&mut parser).expect("bad parse").compile(&mut builder, Local::Scratch);

	builder.build().run(Args::default())
}

macro_rules! run {
	($code:literal) => {
		run_code($code).unwrap()
	};
}

#[test]
fn divides() {
	let result = run!(
		r#"
			Integer.zero? = n -> { n == 0 };
			Integer.divides? = (n, l) -> { (l % n).zero?() };
			12.divides?(24).and(!12.divides?(13))
		"#
	);

	assert_eq!(result.downcast::<Boolean>().unwrap(), true);
}

#[test]
fn square_root() {
	let result = run!(
		r#"
			Integer.'^' = Integer::'**';
			Integer.'√' = n -> { n ^ 0.5 };
			√16
		"#
	);

	assert_eq!(result.downcast::<Float>().unwrap(), 4.0);
}

#[test]
fn fib_set_attr() {
	// NOTE: I'm not sure these semantics are what we want, ie setting an attr on the function means
	// the block its in inherits those attrs.
	let result = run!(
		r#"
			fib = n -> {
				(n <= 1).then(n.return);

				fibb(n - 1) + fibb(n - 2)
			};

			fib.fibb = fib;
			fib(10)
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn fib_set_parent() {
	// NOTE: This won't be necessary later when i get auto inheritance working.
	let result = run!(
		r#"
			fib = n -> {
				(n <= 1).then(n.return);

				fib(n - 1) + fib(n - 2)
			};

			fib.__parents__ = [:0];
			fib(10)
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn fib_pass_function() {
	let result = run!(
		r#"
			fib = (n, fn) -> {
				(n <= 1).then(n.return);

				fn(n - 1, fn) + fn(n - 2, fn)
			};

			fib(10, fib)
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
fn fib_normal() {
	let result = run!(
		r#"
			fib = n -> {
				(n <= 1).then(n.return);

				fib(n - 1) + fib(n - 2)
			};

			fib(10)
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 55);
}

#[test]
#[ignore] // This is currently a bug
fn modifying_string_literals_isnt_global() {
	let result = run!(
		r#"
			modify = { "x".concat("y") };

			modify() + modify()
		"#
	);

	assert_eq!(*result.downcast::<Gc<Text>>().unwrap().as_ref().unwrap(), "xyxy");
}

#[test]
fn assign_and_fetch_from_arrays() {
	let result = run!(
		r#"
			ary = [9, 12, -99];
			ary[1] = 4;
			ary[0] + ary[1]
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 13);
}

#[test]
fn if_and_while() {
	let result = run!(
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
		"#
	);

	assert_eq!(result.downcast::<Integer>().unwrap(), 20);
}

#[test]
fn basic_stackframe_continuation() {
	let result = run!(
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
		"#
	);

	assert_eq!(
		result.downcast::<Gc<Text>>().unwrap().as_ref().unwrap().as_str(),
		"X:XX:XXX:XXXX"
	);
}

#[test]
fn dbg_representations() {
	let result = run!(
		r#"
			block = { :0 };
			frame = block();
			[true, false, null, 12."+", 1.12, 1, "f\n", frame, block].dbg()
		"#
	);

	// We don't actually check the return value as it's not defined exactly.
	assert!(result.is_a::<Gc<Text>>());

	// Also note we don't test `Integer` and friends debug representations. See issue #23
}

#[test]
fn basic_syntax() {
	let result = run!(
		r#"
			$syntax { 12 $bar:(3 $| 4) } = { 12 - $bar };
			12 3
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 9);

	let result = run!(
		r#"
			$syntax { 12 $bar:(3 $| 4) } = { 12 - $bar };
			12 4
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 8);
}

#[test]
fn nested_syntax() {
	let result = run!(
		r#"
			$syntax { defn $name:(a $| b) } = {
				$$syntax { $name } = { 3 - };
			};

			defn a
			(a 10) * (a 0)
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), -21);
}

#[test]
fn if_while_and_do_while() {
	let result = run!(
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
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 25);
}

#[test]
fn alias_macro() {
	let result = run!(
		r#"
			$syntax { alias $new:token $orig:token ; } = {
			  $$syntax { $new } = { $orig };
			};

			alias <- = ;
			alias __current_stackframe__ :0 ;
			x <- 3;
			__current_stackframe__.x
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 3);
}

#[test]
fn list_comprehension() {
	let result = run!(
		r#"
			$syntax {
				[ $body:tt | $var:ident in $src:tt ]
			} = {
				$src.map($var -> { $body })
			};

			[(x * 2) | x in [1,2,3,4]]
		"#
	);

	let list = result.downcast::<Gc<List>>().unwrap().as_ref().unwrap();
	assert_eq!(list.len(), 4);
	assert_eq!(list.as_slice()[0].downcast::<Integer>().unwrap(), 2);
	assert_eq!(list.as_slice()[1].downcast::<Integer>().unwrap(), 4);
	assert_eq!(list.as_slice()[2].downcast::<Integer>().unwrap(), 6);
	assert_eq!(list.as_slice()[3].downcast::<Integer>().unwrap(), 8);
}

#[test]
fn lists_containing_themselves() {
	let result = run!(
		r#"
			l = [1,2];
			l[0] = [3,4];
			l[0][0] = l[0];
			l[1] = l[0];
			l.dbg()
		"#
	);

	let result = result.downcast::<Gc<Text>>().unwrap().as_ref().unwrap();
	assert_eq!(result.as_str(), "[[[...], 4], [[...], 4]]");
}

#[test]
fn reference_syntax_groups() {
	let result = run!(
		r#"
			$syntax time { $hr:int : $min:int } = { $hr : $min . 0 } ;
			$syntax time { $hr:int : $min:int . $sec:int } = { (($min*60) + ($hr*3600) + $sec) } ;

			$syntax { $t:time am } = { $t } ;
			$syntax { $t:time pm } = { ($t + 216_000) } ;

			(10 : 30 . 45 pm) - (10 : 30 am)
		"#
	);
	let ten_thirty_fourtyfive_pm = (10 * 3600 + 30 * 60 + 45) + 216000;
	let ten_thirty_am = 10 * 3600 + 30 * 60;
	assert_eq!(result.downcast::<Integer>().unwrap(), ten_thirty_fourtyfive_pm - ten_thirty_am);
}

#[test]
fn any_parens_in_syntax() {
	let result = run!(
		r#"
			$syntax ( a ) = ( 2 );
			$syntax ( b ) = [ 3 ];
			$syntax ( c ) = { 5 };

			$syntax [ d ] = ( 7 );
			$syntax [ e ] = [ 11 ];
			$syntax [ f ] = { 13 };

			$syntax { g } = ( 17 );
			$syntax { h } = [ 19 ];
			$syntax { i } = { 23 };

			a * b * c * d * e * f * g * h * i
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 223092870);
}

#[test]
fn repetition_in_macros() {
	let result = run!(
		r#"
			$syntax {
				if $cond:group $body:block
				${ else if $cond1:group $body1:block }
				$[ else $body2:block ]
			} = {
				if_cascade($cond, $body ${, {$cond1}, $body1 } $[, $body2])
			};

			x = 2;
			if (x == 0) { 10 }
			else if (x == 1) { 20 }
			else if (x == 2) { 30 }
			else { 40 }
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 30);
}

#[test]
fn negative_matches_and_underscore() {
	let result = run!(
		r#"
			$syntax end { end $| END } = { end } ;
			$syntax { begin ${ $! $_:end $x:token} $_:end } = {
				${$x *} 1
			};

			begin 2 5 7 END
		"#
	);
	assert_eq!(result.downcast::<Integer>().unwrap(), 70);
}

#[test]
fn create_frame_iteration() {
	let result = run!(
		r#"
			iter = acc -> {
			  acc = acc + "X";
			  acc
			};

			frame = iter.create_frame("");
			[frame.restart(),
			 frame.restart(),
			 frame.restart(),
			 frame.restart()].join(":")
		"#
	)
	.downcast::<Gc<Text>>()
	.unwrap();

	assert_eq!(*result.as_ref().unwrap(), "X:XX:XXX:XXXX");
}

#[test]
fn should_overflow_and_return_error() {
	let result = run_code(
		r#"
			{ __block__() }()
		"#,
	)
	.unwrap_err();

	assert!(
		matches!(result.kind, quest::ErrorKind::StackOverflow),
		"didnt overflow, but {:?}",
		result.kind
	);
}
