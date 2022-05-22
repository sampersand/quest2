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
  | (? any non-syntax token ?)
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

		while ReplacementAtom::attempt_to_parse(&mut body, parser, end)? {
			// do nothing
		}

		if parser.take_if_contents_bypass_syntax(TokenContents::RightParen(end))?.is_none() {
			return Err(parser.error(format!("expected `{:?}` after replacement body", end).into()));
		}

		Ok(Self(body))
	}
}

impl<'a> ReplacementAtom<'a> {
	pub fn attempt_to_parse(seq: &mut Vec<Self>, parser: &mut Parser<'a>, end: ParenType) -> Result<'a, bool> {
		match parser.take_bypass_syntax()? {
			Some(Token { contents: TokenContents::SyntaxIdentifier(0, name), .. }) => {
				seq.push(Self::Capture(name));
				Ok(true)
			},
			Some(Token { contents: TokenContents::SyntaxIdentifier(n, name), span }) => {
				seq.push(Self::Token(Token { contents: TokenContents::SyntaxIdentifier(n - 1, name), span }));
				Ok(true)
			},

			Some(Token { contents: TokenContents::SyntaxLeftParen(0, paren), .. }) => {
				seq.push(Self::Paren(paren, ReplacementBody::parse(parser, paren)?));
				Ok(true)
			},
			Some(Token { contents: TokenContents::SyntaxLeftParen(n, paren), span }) => {
				seq.push(Self::Token(Token { contents: TokenContents::SyntaxLeftParen(n - 1, paren), span }));
				Ok(true)
			},

			Some(Token { contents: TokenContents::SyntaxOr(0), .. }) => unreachable!(),
			Some(Token { contents: TokenContents::SyntaxOr(n), span }) => {
				seq.push(Self::Token(Token { contents: TokenContents::SyntaxOr(n - 1), span }));
				Ok(true)
			},

			Some(left @ Token { contents: TokenContents::LeftParen(paren), .. }) => {
				seq.push(Self::Token(left));

				while Self::attempt_to_parse(seq, parser, paren)? {
					// do nothing
				}
				if let Some(right) = parser.take_if_contents_bypass_syntax(TokenContents::RightParen(paren))? {
					seq.push(Self::Token(right));
					Ok(true)
				} else {
					Err(left.span.start.error("parens in syntax must be matched!".to_string().into()))
				}
			},

			Some(token @ Token { contents: TokenContents::RightParen(paren), .. }) if paren == end => {
				parser.untake(token);
				Ok(false)
			},
			Some(token) => {
				seq.push(Self::Token(token));
				Ok(true)
			},
			None => Ok(false)
		}
	}
}


impl<'a> Replacement<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser.take_if_contents_bypass_syntax(TokenContents::LeftParen(ParenType::Curly))?.is_none() {
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
		for atom in self.0.iter().rev() {
			atom.replace(matches, parser)?;
		}

		Ok(())
	}
}

impl<'a> ReplacementAtom<'a> {
	fn replace(&self, matches: &mut PatternMatches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		// TODO: remove 1 from every syntax token here.

		match self {
			Self::Token(token) => {
				parser.untake(*token);
				Ok(())
			},
			Self::Capture(name) => {
				let captures = matches.capture(name)
					.ok_or_else(|| parser.error(format!("syntax variable ${} never matched", name).into()))?;

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

