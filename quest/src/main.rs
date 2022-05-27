#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use quest;
use quest::parse::ast::Compile;
use quest::parse::{token::*, *};
use quest::value::ty::*;
use quest::value::*;
use quest::vm::*;
use quest::Result;

fn run_code(code: &str) -> Result<AnyValue> {
	let mut parser = Parser::new(code, None);
	let mut builder = quest::vm::block::Builder::new(Default::default(), None);
	let scratch = quest::vm::block::Local::Scratch;

	ast::Group::parse_all(&mut parser)
		.expect("bad parse")
		.compile(&mut builder, scratch);

	builder.build().run(Default::default())
}

fn setup_tracing() {
	use tracing::level_filters::LevelFilter;
	use tracing_subscriber::{layer::SubscriberExt, registry::Registry};

	let loglevel = std::env::var("QUEST_LOGGING");
	let filter = match loglevel.as_ref().map(|x| x.as_ref()) {
		Ok("T") | Ok("TRACE") => LevelFilter::TRACE,
		Ok("D") | Ok("DEBUG") => LevelFilter::DEBUG,
		Ok("I") | Ok("INFO") => LevelFilter::INFO,
		Ok("W") | Ok("WARN") => LevelFilter::WARN,
		Ok("E") | Ok("ERROR") => LevelFilter::ERROR,
		Ok("O") | Ok("OFF") => LevelFilter::OFF,
		_ => LevelFilter::WARN,
	};

	tracing_subscriber::fmt()
		.with_max_level(filter)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
		.init();
}

fn main() {
	setup_tracing();
	if false {
		run_code(
			r##"
__EOF__
	; = x 0
	: W ! ? x 10
		; O x
		: = x + x 1
XDONE
__EOF__
$syntax end { end $| END } = { end } ;
$syntax { begin ${ $! $_:end $x:token} $_:end } = {
	${print($x);}
};

begin 1 2 3 END
__EOF__
$syntax {
	object
		# parents
		$[(
			$[$parent1:tt ${, $parent_rest:tt} $[,]]
		)]
	{
		# attributes
		${ $key:literal = $value:tt ; }
	}
} = {
	({
		$[ __parents__ = [$parent1 ${,$parent_rest}]; ]
		${ $key = $value; }
		:0
	}())
} ;

l = object { m = 4; };
person = object (l) {
	n = 3;
	x = (a -> { a.m - a.n });
};

print(person.x());

__EOF__
# $syntax { fn $name:ident = $body:block } = { $name = $body } ;
$syntax { %% $b:block } = { $b };
$syntax { %% $a:ident ${$r:ident} $b:block } = ( { $a -> %% ${$r} $b } );
$syntax { fn $name:ident ${$rest:ident} $body:block } = { $name = (%% ${ $rest } $body)() ; };
$syntax ( @ $fn:ident ${$arg:int} ) = ( ($fn ${($arg)}) );

fn add a b { a + b }
!print (!add 1 2) #=> 3

__EOF__
print(add(1)(2));
#print(x(2))
# add = a -> { b -> { a + b } };
# a = add(1)(2);
# print(a);
# 
# __EOF__
$syntax { %% $body:block } = { $body };
$syntax { %% $a:ident ${$rest:ident} $body:block } = (
	{ $a -> %% ${$rest} $body }
);
$syntax { $$ a } = { exit(0); } ;
$syntax { fn $name:ident = $body:block } = { $name = $body } ;
$syntax {
	fn $name:ident $init:ident ${$rest:ident} = $body:block
} = {
	$name = $init -> %% ${ $arg } $body ;
};

fn add a b c d = { a }

# add = a -> { b -> { a + b } };
print(add(1));
__EOF__
print(add(1)(2));
#print(x(2))
__EOF__
$syntax { @ $e:text } = { stack.push($e); };
$syntax { @ ++ } = { a = stack.pop(); stack.push(stack.pop() + a); };
$syntax { @ . } = { print(stack.pop()); };
$syntax { @ x } = { a = stack.pop(); b = stack.pop(); stack.push(a); stack.push(b); } ;
$syntax { begin ${$tkn:token} } = { stack = []; ${@ $tkn} };
begin

"Hello," " " ++ "world!" x . . #=> Hello,<newline>world!
__EOF__

if_cascade(
	x == 0, { print("x = 0") },
	{ x == 1 }, {print("x = 1"); },
	{ x == 2 }, { print("x = 2"); },
	{ print("x is something else"); });

$syntax {
	if $cond:group $body:block
	${ else if $cond1:group $body1:block }
	$[ else $body2:block ]
} = {
	if_cascade($cond, $body ${, {$cond1}, $body1 } $[, $body2])
};

x = 3;
if (x == 0) { 10 }
else if (x == 1) { 20 }
else if (x == 2) { 30 }
else { 40 }
__EOF__
$syntax { @ $e:literal } = { stack.push($e); };
$syntax { @ . } = { print(stack.pop()); };
$syntax { begin ${$tkn:token} } = { stack = []; ${@ $tkn} };
begin

"hello" "sup" "quest syntax rocks" . . .

__EOF__
$syntax { @ ${! $f:int} } = { print(2 ${* $f}) } ;

@ !3 !5 !7
"##,
		)
		.unwrap();
		return;
	}

	match run_code(&std::env::args().skip(1).next().expect("usage: <expr>")) {
		Err(err) => {
			eprintln!("error: {}", err);
			std::process::exit(0)
		},
		Ok(num) => {
			if let Some(exit_code) = num.downcast::<i64>() {
				std::process::exit(exit_code as i32)
			}
		},
	}
}
