#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use quest;
use quest::parser::ast::Compile;
use quest::parser::{pattern::*, token::*, *};
use quest::value::ty::*;
use quest::value::*;
use quest::vm::*;
use quest::Result;


fn run_code(code: &str) -> Result<AnyValue> {
	let mut parser = Parser::new(code, None);
	let mut builder = quest::vm::block::Builder::new(quest::vm::SourceLocation {}, None);
	let scratch = builder.scratch();

	ast::Group::parse_all(&mut parser)
		.expect("bad parse")
		.compile(&mut builder, scratch);

	builder.build().run(Default::default())
}

fn setup_tracing() {
	use tracing::level_filters::LevelFilter;
	use tracing_subscriber::{layer::SubscriberExt, registry::Registry};

	let loglevel = std::env::var("QUEST_LOGGING");
	let filter = 
		match loglevel.as_ref().map(|x| x.as_ref()) {
			Ok("T") | Ok("TRACE") => LevelFilter::TRACE,
			Ok("D") | Ok("DEBUG") => LevelFilter::DEBUG,
			Ok("I") | Ok("INFO") => LevelFilter::INFO,
			Ok("W") | Ok("WARN") => LevelFilter::WARN,
			Ok("E") | Ok("ERROR") => LevelFilter::ERROR,
			Ok("O") | Ok("OFF") => LevelFilter::OFF,
			_ => return
		};

	tracing_subscriber::fmt()
		.with_max_level(filter)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
		.init();
}


fn main() {
	setup_tracing();
	tracing::debug!("created str constant");

// count = 0;
// while2 = (a, b) -> { a; b; if(a(), b) };
// recur = {
// 	while2({ true }, {
// 		:-1.count = count + 1;
// 		:1.return(:-2);
// 	})
// };
// tmp = recur();
// print(count);
// tmp = tmp.resume();
// print(count);
// tmp = tmp.resume();
// print(count);
	run_code(r#"

recur = acc -> {
	[acc, :0].return();

	0; 0; # for some reason you need two blank statements?

	recur(acc + "X")
};

tmp = recur(""); print(tmp[0]);       #=>X
tmp = tmp[1].resume(); print(tmp[0]); #=>XX
tmp = tmp[1].resume(); print(tmp[0]); #=>XXX
tmp = tmp[1].resume(); print(tmp[0]); #=>XXXX
__EOF__
print(s);
tmp = tmp.resume();
print(s);
tmp = tmp.resume();
print(s);

__EOF__
recur = acc -> {
	acc.concat("X");
	:0.return();

	0; 0;

	recur(acc)
};

s = "";
tmp = recur(s);
print(s);
tmp = tmp.resume();
print(s);
tmp = tmp.resume();
print(s);
	"#).unwrap();
	return;


	match run_code(&std::env::args().skip(1).next().unwrap()) {
		Err(err) => { eprintln!("error: {}", err); std::process::exit(0) },
		Ok(num) => if let Some(exit_code) = num.downcast::<i64>() { std::process::exit(exit_code as i32) }
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

// 		builder.constant(1.as_any(), one);
// 		builder.constant("then".as_any(), tmp2);
// 		builder.constant("return".as_any(), scratch);
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
// 		.set_attr("fib".as_any(), fib.as_any())?;

// 	let fib_of = 30;
// 	let result = fib.run(Args::new(&[fib_of.as_any()], &[]))?;

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
