#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use quest;
use quest::value::ty::*;
use quest::value::*;
use quest::vm::*;
use quest::Result;

fn main() -> Result<()> {
	let mut stream = quest::parser::Stream::new(
		r###"
$syntax {
	if $cond:expr $body:block $[ else $else:block ]
} = {
	Kernel::if($cond, $body $[, $else])
};
__EOF__
add = fn (bar, baz) {
	return [bar ++ baz, "3\n4", 'yu\tp', 4, true]
}
"###,
		None,
	);

	while let Some(tkn) = quest::parser::Token::parse(&mut stream).unwrap() {
		println!("{:?}", tkn);
	}

	Ok(())
}
