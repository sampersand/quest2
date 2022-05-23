use crate::parse::token::{Token, TokenContents, ParenType};
use crate::parse::{Parser, Result};
use std::collections::HashMap;
/*

(*
 NOTE: `'$'*FOO` means that `FOO` can be proceeded by any amount of `$`s.
 Likewise, `'$'+FOO` must be proceeded by at least one `$`
*)
pattern := '{' pattern-body '}' ;
pattern-body := pattern-sequence {'|' pattern-sequence} ;
pattern-sequence := pattern-atom {pattern-atom};
pattern-atom
 := '$'+ident ':' pattern-kind
  | '$'*( pattern-body ')' (* note that non-`$` braces have to be matched *)
  | '$'*[ pattern-body ']'
  | '$'*{ pattern-body '}'
  | (? any non-syntax token ?)
  ;
pattern-kind := ident | '(' pattern-body ')' ;
*/

#[derive(Debug)]
pub struct Pattern<'a>(PatternBody<'a>);

#[derive(Debug)]
struct PatternBody<'a>(Vec<PatternSequence<'a>>);

#[derive(Debug)]
struct PatternSequence<'a>(Vec<PatternAtom<'a>>);

#[derive(Debug)]
enum PatternKind<'a> {
	Name(&'a str),
	Body(ParenType, PatternBody<'a>)
}

#[derive(Debug)]
enum PatternAtom<'a> {
	Capture(&'a str, PatternKind<'a>),
	Paren(ParenType, PatternBody<'a>),
	Token(Token<'a>)
}

impl<'a> PatternBody<'a> {
	fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Option<Self>> {
		let mut body = if let Some(seq) = PatternSequence::parse(parser, end)? {
				vec![seq]
			} else {
				return Ok(None)
			};

		while parser.take_if_contents_bypass_syntax(TokenContents::SyntaxOr(0))?.is_some() {
			if let Some(seq) = PatternSequence::parse(parser, end)? {
				body.push(seq);
			} else {
				return Err(parser.error("expected pattern sequence after `|`".to_string().into()));
			}
		}

		if parser.take_if_contents_bypass_syntax(TokenContents::RightParen(end))?.is_none() {
			return Err(parser.error(format!("expected `{:?}` after pattern body", end).into()));
		}

		Ok(Some(Self(body)))
	}
}

impl<'a> PatternSequence<'a> {
	fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Option<Self>> {
		let mut seq = Vec::new();

		while !matches!(parser.peek_bypass_syntax()?, Some(Token { contents: TokenContents::SyntaxOr(0), .. })) {
			if !PatternAtom::attempt_to_parse(&mut seq, parser, end)? {
				break;
			}
		}

		if seq.is_empty() {
			Ok(None)
		} else {
			Ok(Some(Self(seq)))
		}
	}
}

impl<'a> PatternKind<'a> {
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		match parser.take_bypass_syntax()? {
			Some(Token { contents: TokenContents::Identifier(name), .. }) => Ok(Some(Self::Name(name))),
			Some(Token { contents: TokenContents::LeftParen(paren), .. }) => {
				if let Some(body) = PatternBody::parse(parser, paren)? {
					Ok(Some(Self::Body(paren, body)))
				} else {
					Err(parser.error(format!("expected {:?} pattern body", paren).into()))
				}
			},
			Some(token) => {
				parser.untake(token);
				Ok(None)
			}
			None => Ok(None),
		}
	}
}

impl<'a> PatternAtom<'a> {
	fn attempt_to_parse(seq: &mut Vec<Self>, parser: &mut Parser<'a>, end: ParenType) -> Result<'a, bool> {
		match parser.take_bypass_syntax()? {
			// `$foo` should be followed via `:` and a `kind`
			Some(Token { contents: TokenContents::SyntaxIdentifier(0, name), .. }) => {
				if parser.take_if_contents_bypass_syntax(TokenContents::Symbol(":"))?.is_none() {
					return Err(parser.error("you must put a `:` after a syntax name".to_string().into()));
				}

				if let Some(kind) = PatternKind::parse(parser)? {
					seq.push(Self::Capture(name, kind));
					Ok(true)
				} else {
					Err(parser.error("expected syntax kind after `:`".to_string().into()))
				}
			},
			Some(Token { contents: TokenContents::SyntaxLeftParen(0, paren), .. }) => {
				if let Some(body) = PatternBody::parse(parser, paren)? {
					seq.push(Self::Paren(paren, body));
					Ok(true)
				} else {
					Err(parser.error(format!("expected syntax body after $`{:?}`", paren).into()))
				}
			},

			Some(token @ Token { contents: TokenContents::SyntaxIdentifier(..), .. }
			   | token @ Token { contents: TokenContents::SyntaxOr(..), .. }
			   | token @ Token { contents: TokenContents::SyntaxLeftParen(..), .. }
			) => {
				parser.untake(token);
				Ok(false)
			},

			// TODO: handle matched parens so we can have `$syntax { foo { } }` within our code.
			// Some(token @ Token { contents: TokenContents::RightParen(rp), .. }) if rp == end=> {
			// 	parser.untake(token);
			// 	Ok(None)
			// },

			Some(left @ Token { contents: TokenContents::LeftParen(paren), .. }) => {
				seq.push(Self::Token(left));

				while Self::attempt_to_parse(seq, parser, paren)? {
					// do nothing
				}

				if let Some(right) = parser.take_if_contents_bypass_syntax(TokenContents::RightParen(paren))? {
					seq.push(Self::Token(right));
					Ok(true)
				} else {
					Err(left.span.start.error("parens in syntaxes must be matched!".to_string().into()))
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

impl<'a> Pattern<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser.take_if_contents_bypass_syntax(TokenContents::LeftParen(ParenType::Curly))?.is_none() {
			return Ok(None);
		}

		let body = 
			if let Some(body) = PatternBody::parse(parser, ParenType::Curly)? {
				body
			} else {
				return Err(parser.error("you cannot create empty syntax matches".to_string().into()));
			};

		Ok(Some(Self(body)))
	}
}

#[derive(Debug, Default)]
pub struct PatternMatches<'a> {
	all_tokens: Vec<Token<'a>>,
	captures: HashMap<&'a str, Vec<PatternMatches<'a>>>,
}

impl<'a> Pattern<'a> {
	pub fn matches(&self, parser: &mut Parser<'a>) -> Result<'a, Option<PatternMatches<'a>>> {
		self.0.matches(parser)
	}
}

impl<'a> PatternBody<'a> {
	fn matches(&self, parser: &mut Parser<'a>) -> Result<'a, Option<PatternMatches<'a>>> {
		self.0
			.iter()
			.find_map(|sequence| sequence.matches(parser).transpose())
			.transpose()
	}
}

impl<'a> PatternSequence<'a> {
	fn matches(&self, parser: &mut Parser<'a>) -> Result<'a, Option<PatternMatches<'a>>> {
		let mut matches = PatternMatches::default();

		for atom in &self.0 {
			if !atom.does_match(&mut matches, parser)? {
				parser.untake_tokens(matches.all_tokens);
				return Ok(None)
			}
		}

		Ok(Some(matches))
	}
}

impl<'a> PatternAtom<'a> {
	fn does_match(&self, matches: &mut PatternMatches<'a>, parser: &mut Parser<'a>) -> Result<'a, bool> {
		match self {
			Self::Capture(name, PatternKind::Body(ParenType::Round, body)) => {
				if let Some(new_matches) = body.matches(parser)? {
					let x = matches.captures.insert(name, vec![new_matches]);
					if x.is_some() {
						panic!("duplicate syntax encountered. (todo, error?)");
					}
					Ok(true)
				} else {
					Ok(false)
				}
			},
			Self::Capture(_, _) => todo!(),
			Self::Paren(_, _) => todo!(),
			Self::Token(token) => {
				if let Some(token) = parser.take_if_contents_bypass_syntax(token.contents)? {
					matches.all_tokens.push(token);
					Ok(true)
				} else {
					Ok(false)
				}
			},
		}
	}
}

impl<'a> PatternMatches<'a> {
	pub fn all_tokens(&self) -> &[Token<'a>] {
		&self.all_tokens
	}

	pub fn capture(&self, name: &str) -> Option<&[PatternMatches<'a>]> {
		self.captures.get(name).map(Vec::as_slice)
	}
}