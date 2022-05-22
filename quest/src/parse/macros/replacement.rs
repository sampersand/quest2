use crate::parse::token::{Token, TokenContents, ParenType};
use crate::parse::{Parser, Result};
use super::pattern::PatternMatches;

/*
replacements := '{' replacement-body '}'
replacement-body := replacement-atom {replacement-atom} ;
replacement-atom
 := '$'+ident
  | '$'+[ pattern-body ']'
  | '$'+( pattern-body ')'
  | '$'+{ pattern-body '}'
  | (? any non-macro token ?)
  ;
*/

#[derive(Debug)]
pub struct Replacement<'a>(ReplacementBody<'a>);

#[derive(Debug)]
struct ReplacementBody<'a>(Vec<ReplacementAtom<'a>>);

#[derive(Debug)]
enum ReplacementAtom<'a> {
	Capture(&'a str),
	Paren(ParenType, ReplacementBody<'a>),
	Token(Token<'a>)
}

impl<'a> ReplacementBody<'a> {
	pub fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Self> {
		let mut body = Vec::new();

		while let Some(atom) = ReplacementAtom::parse(parser, end)? {
			body.push(atom)
		}

		if parser.take_if_contents_bypass_macros(TokenContents::RightParen(end))?.is_none() {
			return Err(parser.error(format!("expected `{:?}` after replacement body", end).into()));
		}

		Ok(Self(body))
	}
}

impl<'a> ReplacementAtom<'a> {
	pub fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Option<Self>> {
		match parser.take_bypass_macros()? {
			Some(Token { contents: TokenContents::MacroIdentifier(0, name), .. }) => {
				Ok(Some(Self::Capture(name)))
			},
			Some(Token { contents: TokenContents::MacroIdentifier(n, name), span }) => {
				Ok(Some(Self::Token(Token { contents: TokenContents::MacroIdentifier(n - 1, name), span })))
			},

			Some(Token { contents: TokenContents::MacroLeftParen(0, paren), .. }) => {
				Ok(Some(Self::Paren(paren, ReplacementBody::parse(parser, paren)?)))
			},
			Some(Token { contents: TokenContents::MacroLeftParen(n, paren), span }) => {
				Ok(Some(Self::Token(Token { contents: TokenContents::MacroLeftParen(n - 1, paren), span })))
			},

			// TODO: handle matched parens so we can have `$syntax { foo { } }` within our code.
			// Some(token @ Token { contents: TokenContents::RightParen(rp), .. }) if rp == end=> {
			// 	parser.untake(token);
			// 	Ok(None)
			// },

			Some(token @ Token { contents: TokenContents::RightParen(rp), .. }) if rp == end => {
				parser.untake(token);
				Ok(None)
			},
			Some(token) => Ok(Some(Self::Token(token))),
			None => Ok(None)
		}
	}
}


impl<'a> Replacement<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser.take_if_contents_bypass_macros(TokenContents::LeftParen(ParenType::Curly))?.is_none() {
			return Ok(None);
		}

		let body = ReplacementBody::parse(parser, ParenType::Curly)?;
		Ok(Some(Self(body)))
	}
}

impl<'a> Replacement<'a> {
	pub fn replace(&self, mut matches: PatternMatches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		self.0.replace(&mut matches, parser)
	}
}

impl<'a> ReplacementBody<'a> {
	fn replace(&self, matches: &mut PatternMatches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		for atom in self.0.iter() {
			atom.replace(matches, parser)?;
		}

		Ok(())
	}
}

impl<'a> ReplacementAtom<'a> {
	fn replace(&self, matches: &mut PatternMatches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		match self {
			Self::Token(token) => {
				parser.untake(*token);
				Ok(())
			},
			Self::Capture(name) => {
				let captures = matches.capture(name)
					.ok_or_else(|| parser.error(format!("macro variable ${} never matched", name).into()))?;

				for capture in captures {
					parser.untake_tokens(capture.all_tokens().iter().copied());
				}

				Ok(())
			},
			Self::Paren(_kind, _body) => {
				todo!()
			},
		}
	}
}

