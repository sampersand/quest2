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
use std::path::Path;

fn run_code(code: &str, filename: Option<&Path>) -> Result<Value> {
	let mut parser = Parser::new(code, filename);
	let mut builder = quest::vm::block::Builder::new(0, Default::default());
	let scratch = quest::vm::block::Local::Scratch;

	ast::Group::parse_all(&mut parser).expect("bad parse").compile(&mut builder, scratch);

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
			r#"
i=0;
while({ i < 10_000_000 }, {
	:1.i = i + 1;
});
print(i);
__EOF__
Text = "".__parents__[0];
Boolean = true.__parents__[0];
Null = null.__parents__[0];

Integer.tobool = n -> { n != 0 };
Text.tobool = n -> { n != "" };
Boolean.tobool = n -> { n };
Null.tobool = n -> { false };

Integer.tonum = n -> { n };
Text.tonum = n -> { abort("<todo: string to num>") };
Boolean.tonum = n -> { ifl(n, 1, 0) };
Null.tonum = n -> { 0 };

Integer.tostr = n -> { n."@text"() };
Text.tostr = n -> { n."@text"() };
Boolean.tostr = n -> { n."@text"() };
Null.tostr = n -> { n."@text"() };

Integer.add = (l,r) -> { l + r.tonum() };
Text.add = (l,r) -> { l + r.tostr() };
Integer.'^' = Integer::'**';

Integer.lt = (l,r) -> { l < r.tonum() };
Integer.gt = (l,r) -> { l > r.tonum() };
Text.lt = (l,r) -> { l < r.tostr() };
Text.gt = (l,r) -> { l > r.tostr() };
Boolean.lt = (l,r) -> { (!l).and(r.tobool()) };
Boolean.gt = (l,r) -> { l.and(!r.tobool()) };

# whitespace
$syntax kn { @ $_:(\( $| \) $| \[ $| \] $| \{ $| \} $| :) $r:kn } = { $r };

# primitives
$syntax kn { @ $n:int } = { $n };
$syntax kn { @ $n:text } = { $n };
$syntax kn { @ $n:ident } = { env . $n };
$syntax kn { @ $(T $| TRUE) } = { true };
$syntax kn { @ F } = { false };
$syntax kn { @ N } = { null };
# nullary
$syntax kn { @ P } = { abort("<todo: make prompt in quest>") };
$syntax kn { @ R } = { abort("<todo: make random in quest>") };

#unary
$syntax kn { @ E $r:kn } = { abort("<eval isnt supported until quest gets it>") };
$syntax kn { @ B $r:kn } = { { $r } };
$syntax kn { @ C $r:kn } = { ($r)() };
$syntax kn { @ ` $r:kn } = { abort("<todo: make ` in quest>") };
$syntax kn { @ Q $r:kn } = { exit(($r)."@num"()) };
$syntax kn { @ ! $r:kn } = { !($r).tobool() };
$syntax kn { @ L $r:kn } = { ($r).len() };
$syntax kn { @ D $r:kn } = { (x=($r); print(x.dbg()); x) };
$syntax kn { @ O $r:kn } = { ($r.print(); null) };
$syntax kn { @ A $r:kn } = { abort("<todo: make ascii in quest>") };
$syntax kn { @ ~ $r:kn } = { -($r).tonum() };

#binary
$syntax kn { @ + $l:kn $r:kn } = { (($l).add($r)) };
$syntax kn { @ $op:(- $| * $| / $| % $| ^) $l:kn $r:kn } = { (($l) $op ($r)) };
$syntax kn { @ < $l:kn $r:kn } = { (($l).lt($r)) };
$syntax kn { @ > $l:kn $r:kn } = { (($l).gt($r)) };
$syntax kn { @ ? $l:kn $r:kn } = { (($l) == ($r)) };
$syntax kn { @ & $l:kn $r:kn } = { (x=($l); if(x.tobool(), { $r } , { x })) };
$syntax kn { @ | $l:kn $r:kn } = { (x=($l); if(x.tobool(), { x } , { $r })) };
$syntax kn { @ ; $l:kn $r:kn } = { (($l); ($r)) };
$syntax kn { @ = $l:kn $r:kn } = { ($l = ($r)) };
$syntax kn { @ W $l:kn $r:kn } = { (while({ $l }, { $r }); null) };

# ternary
$syntax kn { @ I $l:kn $m:kn $r:kn } = { if($l, { $m }, { $r }) };
$syntax kn { @ G $l:kn $m:kn $r:kn } = { abort("<todo: substr in quest>"); };

# quaternary
$syntax kn { @ S $l:kn $m:kn $r:kn $x:kn } = { abort("<todo substr>") };

$syntax { knight ${$!XDONE $tkn:token} XDONE } = { (env={:0}(); ${@ $tkn}) };

# and here we go, Knight in Quest! (The only caveat is you can only use single letter names)
knight
	; = fizzbuzz B
		; = n 0
		; = max (+ 1 max)
		: W < (= n + 1 n) max
			: O
				: I ! (% n 15) "FizzBuzz"
				: I ! (% n 5)  "Fizz"
				: I ! (% n 3)  "Buzz"
				                n
	; = max 100
	: C fizzbuzz
XDONE

"#,
			None,
		)
		.unwrap();

		return;
	}

	const USAGE: &str = "usage: -e <expr> | -f <file>";

	let mut args = std::env::args().skip(1);
	let (contents, filename) = match &*args.next().expect(USAGE) {
		"-f" => {
			let name = args.next().expect(USAGE);
			(std::fs::read_to_string(&name).expect("cant open file"), Some(name))
		}
		"-e" => (args.next().expect(USAGE), None),
		_ => panic!("{USAGE}"),
	};

	match run_code(&contents, filename.as_deref().map(Path::new)) {
		Err(err) => {
			eprintln!("error: {err:#}");
			std::process::exit(0)
		}
		Ok(num) => {
			if let Some(exit_code) = num.downcast::<i64>() {
				std::process::exit(exit_code as i32)
			}
		}
	}
}
