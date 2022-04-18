pub mod integer;
pub mod text;

use super::{ErrorKind, Result, Span, Stream};
use crate::value::ty::{Float, Integer, Text};
use crate::value::Gc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParenType {
	Round,
	Curly,
	Square,
}
/*
*/
#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
	Text(Gc<Text>),
	Integer(Integer),
	Float(Float),
	Symbol(&'a str), // eg `+`, `**`, etc. user-definable.
	Identifier(&'a str),
	LeftParen(ParenType),
	RightParen(ParenType),
}

#[derive(Debug)]
pub struct SpannedToken<'a> {
	pub token: Token<'a>,
	pub span: Span<'a>,
}

fn strip_whitespace_and_comments(stream: &mut Stream<'_>) {
	loop {
		stream.take_while(char::is_whitespace);

		if stream.take_if_chr('#').is_none() {
			return;
		}

		stream.take_while(|c| c != '\n');
	}
}

// TODO: maybe use unicode?
fn is_able_to_compose_an_operator(c: char) -> bool {
	"~!@$%^&*-=+|\\;:,<.>/?".contains(c)
}

impl<'a> Token<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Self> {
		match stream.peek().expect("we already checked for eof") {
			c if c.is_whitespace() || c == '#' => {
				unreachable!("we already stripped whitespace & comments")
			},
			'(' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Round))
			},
			')' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Round))
			},
			'[' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Square))
			},
			']' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Square))
			},
			'{' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Curly))
			},
			'}' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Curly))
			},
			'\'' | '\"' => text::parse_text(stream).map(Option::unwrap),
			'0'..='9' => integer::parse_integer(stream).map(Option::unwrap), // technically should parse floats too...
			a if a.is_alphabetic() => Ok(Self::Identifier(stream.take_while(char::is_alphanumeric))),
			a if is_able_to_compose_an_operator(a) => {
				Ok(Self::Symbol(stream.take_while(is_able_to_compose_an_operator)))
			},
			other => panic!("todo: return an error for unknown kind {:?}", other),
		}
	}
}

impl<'a> SpannedToken<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Option<Self>> {
		strip_whitespace_and_comments(stream);

		if stream.is_eof() {
			return Ok(None);
		}

		let start = stream.span_start();
		let token = Token::parse(stream)?;

		Ok(Some(Self {
			token,
			span: start.finish(stream),
		}))
	}
}

// // fn next_non_underscore(stream: &mut Stream<'_>) -> Result<'_, char> {
// // 	while stream.peek()?
// // }

// pub fn with_span<'a>(
// 	stream: &mut Stream<'a>,
// 	func: TokenGenerator
// ) -> Result<'a, Option<SpannedToken<'a>>> {
// 	let start = stream.span_start();

// 	func(stream)
// 		.map(|token| token.map(|token| SpannedToken { token, span: start.finish(stream) }))
// }

// impl Token<'_> {
// 	pub fn parse_integer<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<SpannedToken<'a>>> {
// 		with_span(stream, integer::parse_integer)
// 	}

// 	pub fn parse_text<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<SpannedToken<'a>>> {
// 		with_span(stream, text::parse_text)
// 	}
// }

// impl<'a> Token<'a> {
// 	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Option<Token<'a>>> {
// 		// Remove comments and whitespace beforehand.
// 		loop {
// 			stream.take_while(char::is_whitespace);

// 			if stream.take_if_chr('#').is_none() {
// 				break;
// 			}

// 			stream.take_while(|c| c != '\n');
// 		}

// 		with_span(stream, |stream| {
// 			todo!()
// 		})

// 		match stream.peek() {
// 			Some()
// 			None => Ok(None)
// 		}
// 	}
// }
