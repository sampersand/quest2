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
		_ => return,
	};

	tracing_subscriber::fmt()
		.with_max_level(filter)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
		.init();
}

fn main() {
	setup_tracing();
	if true {
		run_code(
			r#"
$syntax { ++ } = { += 1 };
$syntax { $name:tt += } = { $name = $name + };

x = 3;
x++;
print(x) #=> 4
__EOF__
# ```php
# $syntax { alias $new:token $orig:token } = {
#   $$syntax { $new } = { $orig };
# };
# 
# alias <- =;
# x <- 3;
# 
# ``` even better
$syntax { if $cond:group $body:block } = { (if)($cond, $body); };
$syntax { while $cond:group $body:block } = { (while)({ $cond }, $body); };
$syntax { do $body:block while $cond:group } = { $body(); while $cond $body };

x = 0;
i = 0;
do {
	:-1.x = :-1.x + 1;
	if (0 != :-1.x % 2) {
		:-1.i = :-1.i + :-1.x;
#		print("x is odd: " + :-1.x."@text"());
	}
} while (:1.x < 10);

i.print();

__EOF__
$syntax { while $cond:group $body:block } = { (while)({ $cond }, $body); };
$syntax { do $body:block while $cond:group } = { $body(); while $cond $body };

$syntax {
	for (
		$var:ident = $init:num ;;
		$i:ident $op:symbol $max:num ;;
		$j:ident ++
	) $body:block
} = {
	$var = $init;
	while (:1. $var $op $max) {
		$body();
		:1. $var = :1. $var + 1;
	}
};

for (i = 0 ;; i < 10 ;; i++) {
	print(:-1.i);
}


__EOF__
$syntax { defn $name:ident } = {
	$$syntax { $name } = { 3 - };
};

print(a 4); #=> -1
print(a 2); #=> 1
__EOF__

$syntax { doit $bar:(0 $| 2 $| 4 $| 6 $| 8) } = { print('Even!') };
$syntax { doit $bar:(1 $| 3 $| 5 $| 7 $| 9) } = { print('Odd!') };
doit 1;
doit 8;
doit 9;

$syntax { 12 $bar:(3 $| 4) } = { 12 - $bar };
print(12 3);
__EOF__
upto_ten = n -> {
	(n >= 10).then(return);
	forever.i = forever.i + 1;
	:0
};
forever.i = 0;


forever();

"#,
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

/*
// fn main() -> Result<()> {
// 	let fib = {
// 		let mut builder = quest::vm::block::Builder::new(quest::vm::SourceLocation {}, None);

// 		let n = builder.named_local("n");
// 		let fib = builder.named_local("fib");
// 		let one = builder.unnamed_local();
// 		let tmp = builder.unnamed_local();
// 		let tmp2 = builder.unnamed_local();
// 		let ret = builder.unnamed_local();
// 		let scratch = builder.scratch();

// 		builder.constant(1.to_any(), one);
// 		builder.constant("then".to_any(), tmp2);
// 		builder.constant("return".to_any(), scratch);
// 		builder.get_attr(n, scratch, scratch);
// 		builder.less_equal(n, one, tmp);
// 		builder.call_attr_simple(tmp, tmp2, &[scratch], scratch);
// 		builder.subtract(n, one, n);
// 		builder.call_simple(fib, &[n], tmp);
// 		builder.subtract(n, one, n);
// 		builder.call_simple(fib, &[n], scratch);
// 		builder.add(tmp, scratch, scratch);;

// 		builder.build()
// 	};

// 	fib.as_mut()
// 		.unwrap()
// 		.set_attr("fib".to_any(), fib.to_any())?;

// 	let fib_of = 30;
// 	let result = fib.run(Args::new(&[fib_of.to_any()], &[]))?;

// 	println!("fib({:?}) = {:?}", fib_of, result);

// 	Ok(())
// }

fn main() {
	let mut parser = Parser::new(
		r###"

__EOF__
(
# I haven't currently written the assignment parser,
# but you can call `__set_attr__` to do the same thing.
:0.__set_attr__("fib", {
	(_0 <= 1).then(_0.return);

	fib(_0 - 1) + fib(_0 - 2)
});

fib.__set_attr__("fib", fib);

fib(10).print();
)
__EOF__
# print ( 1 + 2 ) ; #, " ", 3 * 4 )
# if (1 == 1, 2,3)
# print([ 12 + 34 ] [ 0 ]);
(
	:0.__set_attr__("a", 3);
	print(a);
)
__EOF__
	{
		print(if(2 == 2, { 2 }, { 4 }));
		print({ a + 1 }(4));
		34.return(:0);
		print({ a + 1 }(4));
	}();
	print("A");
)

"###,
		None,
	);

	let mut builder = quest::vm::block::Builder::new(quest::vm::SourceLocation {}, None);
	let scratch = builder.scratch();

	ast::Group::parse_all(&mut parser)
		.expect("bad parse")
		.compile(&mut builder, scratch);

	let block = builder.build();
	let result = block.run(Default::default()).unwrap();

	println!("result = {:?}", result);
}

fn main1() -> Result<()> {
	let mut parser = Parser::new(
		r###"
if 1:2 { 1:2; 3:4; 5:6; } else { 1:2; }

__EOF__

$syntax {
	if $cond:expr $body:block $[ else $else:block ]
} = {
	Kernel::if($cond, $body $[, $else])
};
add = fn (bar, baz) {
	return [bar ++ baz, "3\n4", 'yu\tp', 4, true]
}
"###,
		None,
	);

	macro_rules! rc {
		($x:expr) => {
			std::rc::Rc::new($x) as std::rc::Rc<dyn Pattern<'static>>
		};
	}

	parser.add_pattern(
		"block".to_string(),
		rc!(Sequence(vec![
			rc!(Exact(TokenContents::LeftParen(ParenType::Curly))),
			rc!(Repeat::new(
				0,
				None,
				rc!(Sequence(vec![
					rc!(NamedPattern("expr")),
					rc!(Exact(TokenContents::Semicolon)),
				]))
			)
			.unwrap()),
			rc!(Exact(TokenContents::RightParen(ParenType::Curly))),
		])),
	);

	/*  int : int */
	let time = rc!(Sequence(vec![rc!(Literal), rc!(Symbol(Some(":"))), rc!(Literal),]));

	parser.add_pattern("expr".to_string(), time);

	let ifelse = Sequence(vec![
		rc!(Identifier(Some("if"))),
		rc!(Capture("cond", rc!(NamedPattern("expr")))),
		rc!(Capture("ift", rc!(NamedPattern("block")))),
		rc!(Optional(rc!(Sequence(vec![
			rc!(Identifier(Some("else"))),
			rc!(Capture("iff", rc!(NamedPattern("block")))),
		])))),
	]);

	dbg!(ifelse.try_match(&mut parser));
	println!("{:?}", parser);
	while let Some(tkn) = parser.advance().unwrap() {
		println!("{:?}", tkn);
	}

	// while let Some(tkn) = Token::parse(&mut stream).unwrap() {
	// 	println!("{:?}", tkn);
	// }

	Ok(())
}
*/
