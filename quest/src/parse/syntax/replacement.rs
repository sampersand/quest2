use super::Matches;
use crate::parse::token::{ParenType, Token, TokenContents};
use crate::parse::{Parser, Result};

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
	Token(Token<'a>),
}

impl<'a> ReplacementBody<'a> {
	pub fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Self> {
		let mut body = Vec::new();

		while ReplacementAtom::attempt_to_parse(&mut body, parser, end)? {
			// do nothing
		}

		if parser
			.take_if_contents_bypass_syntax(TokenContents::RightParen(end))?
			.is_none()
		{
			return Err(parser.error(format!("expected `{:?}` after replacement body", end).into()));
		}

		Ok(Self(body))
	}
}

impl<'a> ReplacementAtom<'a> {
	pub fn attempt_to_parse(
		seq: &mut Vec<Self>,
		parser: &mut Parser<'a>,
		end: ParenType,
	) -> Result<'a, bool> {
		match parser.take_bypass_syntax()? {
			Some(Token {
				contents: TokenContents::SyntaxIdentifier(0, name),
				..
			}) => {
				seq.push(Self::Capture(name));
				Ok(true)
			},
			Some(Token {
				contents: TokenContents::SyntaxIdentifier(n, name),
				span,
			}) => {
				seq.push(Self::Token(Token {
					contents: TokenContents::SyntaxIdentifier(n - 1, name),
					span,
				}));
				Ok(true)
			},

			Some(Token {
				contents: TokenContents::SyntaxLeftParen(0, paren),
				..
			}) => {
				seq.push(Self::Paren(paren, ReplacementBody::parse(parser, paren)?));
				Ok(true)
			},
			Some(Token {
				contents: TokenContents::SyntaxLeftParen(n, paren),
				span,
			}) => {
				seq.push(Self::Token(Token {
					contents: TokenContents::SyntaxLeftParen(n - 1, paren),
					span,
				}));
				Ok(true)
			},

			Some(Token {
				contents: TokenContents::SyntaxOr(0),
				..
			}) => unreachable!(),
			Some(Token {
				contents: TokenContents::SyntaxOr(n),
				span,
			}) => {
				seq.push(Self::Token(Token {
					contents: TokenContents::SyntaxOr(n - 1),
					span,
				}));
				Ok(true)
			},

			Some(
				left @ Token {
					contents: TokenContents::LeftParen(paren),
					..
				},
			) => {
				seq.push(Self::Token(left));

				while Self::attempt_to_parse(seq, parser, paren)? {
					// do nothing
				}
				if let Some(right) =
					parser.take_if_contents_bypass_syntax(TokenContents::RightParen(paren))?
				{
					seq.push(Self::Token(right));
					Ok(true)
				} else {
					Err(
						left
							.span
							.start
							.error("parens in syntax must be matched!".to_string().into()),
					)
				}
			},

			Some(
				token @ Token {
					contents: TokenContents::RightParen(paren),
					..
				},
			) if paren == end => {
				parser.untake(token);
				Ok(false)
			},

			Some(
				Token {
					contents: TokenContents::EscapedLeftParen(paren),
					span
				},
			) => {
				seq.push(Self::Token(Token { contents: TokenContents::LeftParen(paren), span }));
				Ok(true)
			},
			Some(
				Token {
					contents: TokenContents::EscapedRightParen(paren),
					span
				},
			) => {
				seq.push(Self::Token(Token { contents: TokenContents::RightParen(paren), span }));
				Ok(true)
			},
			Some(token) => {
				seq.push(Self::Token(token));
				Ok(true)
			},
			None => Ok(false),
		}
	}
}

impl<'a> Replacement<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let mut paren = ParenType::Round; // irrelevant, it'll be overwritten

		if parser
			.take_if_bypass_syntax(|token| if let TokenContents::LeftParen(lp) = token.contents {
				paren = lp;
				true
			} else {
				false
			})?
			.is_none()
		{
			return Ok(None);
		}

		let body = ReplacementBody::parse(parser, paren)?;
		Ok(Some(Self(body)))
	}
}

impl<'a> Replacement<'a> {
	pub fn replace(
		&self,
		matches: Matches<'a>,
		parser: &mut Parser<'a>,
	) -> Result<'a, ()> {
		self.0.replace(&matches, parser)
	}
}

impl<'a> ReplacementBody<'a> {
	fn replace(&self, matches: &Matches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		for atom in self.0.iter().rev() {
			atom.replace(matches, parser)?;
		}

		Ok(())
	}
}

impl<'a> ReplacementAtom<'a> {
	fn replace(&self, matches: &Matches<'a>, parser: &mut Parser<'a>) -> Result<'a, ()> {
		// TODO: remove 1 from every syntax token here.

		match self {
			Self::Token(token) => {
				parser.untake(*token);
				Ok(())
			},
			Self::Capture(name) => {
				let captures = matches.capture(name).ok_or_else(|| {
					parser.error(format!("syntax variable ${name} never matched").into())
				})?;

				for capture in captures {
					capture.expand(parser)
				}

				Ok(())
			},
			Self::Paren(kind, body) => {
				let (min, max) = match kind {
					ParenType::Round => (1, Some(1)),
					ParenType::Square => (0, Some(1)),
					ParenType::Curly => (0, None),
				};
				let mut submatches = None;

				for atom in &body.0 {
					if let ReplacementAtom::Capture(name) = atom {
						if let Some(subm) = matches.get_submatches_with(name) {
							submatches = Some(subm);
							break;
						}
					}
				}

				if let Some(submatches) = submatches {
					if submatches.len() < min || max.map_or(false, |max| max < submatches.len()) {
						return Err(parser.error(format!("invalid match count (got {}, min={min},max={max:?})",
							submatches.len()).into()))
					}

					for submatch in submatches {
						for caps in submatch.iter().rev() {
							body.replace(caps, parser)?;
						}
					}

					Ok(())
				} else if min == 0 {
					Ok(()) // do nothing, minimum of zero is ok
				} else {
					Err(parser.error("no matches found (todo: source location?)".to_string().into()))
				}
			},
		}
	}
}
