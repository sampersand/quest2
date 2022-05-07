#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use quest;
use quest::parser::{pattern::*, token::*, *};
use quest::value::ty::*;
use quest::value::*;
use quest::vm::*;
use quest::Result;
use quest::parser::ast::Compile;

fn main() {
	let mut parser = Parser::new(r###"
# print ( 1 + 2 ) ; #, " ", 3 * 4 )
# if (1 == 1, 2,3)
# print([ 12 + 34 ] [ 0 ]);
print(if(2 == 2, { 2 }, { 4 }))
print({ a + 1 }(4))

"###, None);

	let mut builder = quest::vm::Block::builder(quest::vm::SourceLocation {});
	let scratch = builder.scratch();

	while let Some(expr) = ast::Expression::parse(&mut parser).expect("bad parse") {
		expr.compile(&mut builder, scratch);
	}

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
