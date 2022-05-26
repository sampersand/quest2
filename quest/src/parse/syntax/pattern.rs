use super::Matcher;
use crate::parse::token::{ParenType, Token, TokenContents};
use crate::parse::{Parser, Result};
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
	Body(ParenType, PatternBody<'a>),
}

#[derive(Debug)]
enum PatternAtom<'a> {
	Capture(&'a str, PatternKind<'a>),
	Paren(ParenType, PatternBody<'a>),
	Token(Token<'a>),
}

impl<'a> PatternBody<'a> {
	fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Option<Self>> {
		let mut body = if let Some(seq) = PatternSequence::parse(parser, end)? {
			vec![seq]
		} else {
			return Ok(None);
		};

		while parser
			.take_if_contents_bypass_syntax(TokenContents::SyntaxOr(0))?
			.is_some()
		{
			if let Some(seq) = PatternSequence::parse(parser, end)? {
				body.push(seq);
			} else {
				return Err(parser.error("expected pattern sequence after `|`".to_string().into()));
			}
		}

		if parser
			.take_if_contents_bypass_syntax(TokenContents::RightParen(end))?
			.is_none()
		{
			return Err(parser.error(format!("expected `{:?}` after pattern body", end).into()));
		}

		Ok(Some(Self(body)))
	}
}

impl<'a> PatternSequence<'a> {
	fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Option<Self>> {
		let mut seq = Vec::new();

		while !matches!(
			parser.peek_bypass_syntax()?,
			Some(Token {
				contents: TokenContents::SyntaxOr(0),
				..
			})
		) {
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
			Some(Token {
				contents: TokenContents::Identifier(name),
				..
			}) => Ok(Some(Self::Name(name))),
			Some(Token {
				contents: TokenContents::LeftParen(paren),
				..
			}) => {
				if let Some(body) = PatternBody::parse(parser, paren)? {
					Ok(Some(Self::Body(paren, body)))
				} else {
					Err(parser.error(format!("expected {:?} pattern body", paren).into()))
				}
			},
			Some(token) => {
				parser.untake(token);
				Ok(None)
			},
			None => Ok(None),
		}
	}
}

impl<'a> PatternAtom<'a> {
	fn attempt_to_parse(
		seq: &mut Vec<Self>,
		parser: &mut Parser<'a>,
		end: ParenType,
	) -> Result<'a, bool> {
		match parser.take_bypass_syntax()? {
			// `$foo` should be followed via `:` and a `kind`
			Some(Token {
				contents: TokenContents::SyntaxIdentifier(0, name),
				..
			}) => {
				if parser
					.take_if_contents_bypass_syntax(TokenContents::Symbol(":"))?
					.is_none()
				{
					return Err(
						parser.error("you must put a `:` after a syntax name".to_string().into()),
					);
				}

				if let Some(kind) = PatternKind::parse(parser)? {
					seq.push(Self::Capture(name, kind));
					Ok(true)
				} else {
					Err(parser.error("expected syntax kind after `:`".to_string().into()))
				}
			},
			Some(Token {
				contents: TokenContents::SyntaxLeftParen(0, paren),
				..
			}) => {
				if let Some(body) = PatternBody::parse(parser, paren)? {
					seq.push(Self::Paren(paren, body));
					Ok(true)
				} else {
					Err(parser.error(format!("expected syntax body after $`{:?}`", paren).into()))
				}
			},

			Some(
				token @ Token {
					contents: TokenContents::SyntaxIdentifier(..),
					..
				}
				| token @ Token {
					contents: TokenContents::SyntaxOr(..),
					..
				}
				| token @ Token {
					contents: TokenContents::SyntaxLeftParen(..),
					..
				},
			) => {
				parser.untake(token);
				Ok(false)
			},

			// TODO: handle matched parens so we can have `$syntax { foo { } }` within our code.
			// Some(token @ Token { contents: TokenContents::RightParen(rp), .. }) if rp == end=> {
			// 	parser.untake(token);
			// 	Ok(None)
			// },
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
							.error("parens in syntaxes must be matched!".to_string().into()),
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

impl<'a> Pattern<'a> {
	pub fn does_match(&self, matcher: &mut Matcher<'a, '_, '_>, parser: &mut Parser<'a>) -> Result<'a, bool> {
		self.0.does_match(matcher, parser)
	}

	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser
			.take_if_contents_bypass_syntax(TokenContents::LeftParen(ParenType::Curly))?
			.is_none()
		{
			return Ok(None);
		}

		let body = if let Some(body) = PatternBody::parse(parser, ParenType::Curly)? {
			body
		} else {
			return Err(parser.error("you cannot create empty syntax matches".to_string().into()));
		};

		Ok(Some(Self(body)))
	}
}

impl<'a> PatternBody<'a> {
	fn does_match(&self, matcher: &mut Matcher<'a, '_, '_>, parser: &mut Parser<'a>) -> Result<'a, bool> {
		for sequence in self.0.iter() {
			if sequence.does_match(matcher, parser)? {
				return Ok(true)
			}
		}
		Ok(false)
	}

	// fn matches(&self, parser: &mut Parser<'a>) -> Result<'a, Option<Matches<'a>>> {
	// 	self
	// 		.0
	// 		.iter()
	// 		.find_map(|sequence| sequence.matches(parser).transpose())
	// 		.transpose()
	// }
}

impl<'a> PatternSequence<'a> {
	fn does_match<'v>(&self, matcher: &mut Matcher<'a, '_, '_>, parser: &mut Parser<'a>) -> Result<'a, bool> {
		let mut submatcher = matcher.submatcher();

		for atom in &self.0 {
			if !atom.does_match(&mut submatcher, parser)? {
				submatcher.unmatch(parser);
				return Ok(false);
			}
		}

		Ok(true)
	}

	// fn matches(&self, parser: &mut Parser<'a>) -> Result<'a, Option<Matches<'a>>> {
	// 	let mut builder = Matcher::default();

	// 	for atom in &self.0 {
	// 		if !atom.does_match(&mut builder, parser)? {
	// 			builder.unmatch(parser);
	// 			return Ok(None);
	// 		}
	// 	}

	// 	Ok(Some(builder.finish()))
	// }
}

fn match_group<'a>(
	matcher: &mut Matcher<'a, '_, '_>,
	parser: &mut Parser<'a>,
	paren: ParenType,
) -> Result<'a, bool> {
	loop {
		let token = if let Some(token) = parser.take_bypass_syntax()? {
			token
		} else {
			return Ok(false);
		};
		matcher.push(token);

		match token.contents {
			TokenContents::RightParen(rp) if rp == paren => return Ok(true),
			TokenContents::LeftParen(lp) => {
				if !match_group(matcher, parser, lp)? {
					return Ok(false);
				}
			},
			_ => {},
		}
	}
}

fn does_match_named<'a>(
	capture_name: &'a str,
	name: &str,
	matcher: &mut Matcher<'a, '_, '_>,
	parser: &mut Parser<'a>,
) -> Result<'a, bool> {
	let mut submatcher = matcher.submatcher();

	macro_rules! single_token_group {
		($pat:pat) => {
			if let Some(token) =
				parser.take_if_bypass_syntax(|token| matches!(token.contents, $pat))?
			{
				submatcher.push(token);
				let subm = submatcher.finish();
				matcher.declare_capture(capture_name, vec![subm])?;
				Ok(true)
			} else {
				Ok(false)
			}
		};
	}

	use TokenContents::{Float, Identifier, Integer, Stackframe, Symbol, Text};

	match name {
		"token" => single_token_group!(_),
		"text" => single_token_group!(Text(_)),
		"int" | "integer" => single_token_group!(Integer(_)),
		"float" => single_token_group!(Float(_)),
		"num" | "number" => single_token_group!(Integer(_) | Float(_)),
		"ident" | "identifier" => single_token_group!(Identifier(_)),
		"stackframe" => single_token_group!(Stackframe(_)),
		"symbol" => single_token_group!(Symbol(_)),
		"literal" => {
			single_token_group!(Integer(_) | Float(_) | Identifier(_) | Text(_) | Stackframe(_))
		},

		"tt" => Ok(does_match_named(capture_name, "literal", matcher, parser)?
			|| does_match_named(capture_name, "group", matcher, parser)?
			|| does_match_named(capture_name, "list", matcher, parser)?
			|| does_match_named(capture_name, "block", matcher, parser)?),
		"group" | "block" | "list" => {

			let paren = match (name, parser.take_bypass_syntax()?) {
				("group", Some(token @ Token {contents: TokenContents::LeftParen(paren @ ParenType::Round), .. }))
				| ("block", Some(token @ Token {contents: TokenContents::LeftParen(paren @ ParenType::Curly), .. }))
				| ("list", Some(token @ Token {contents: TokenContents::LeftParen(paren @ ParenType::Square), .. })) => {
					submatcher.push(token);
					paren
				},
				(_, Some(token)) => {
					parser.untake(token);
					return Ok(false);
				},
				(_, None) => return Ok(false),
			};

			if match_group(&mut submatcher, parser, paren)? {
				let matches = submatcher.finish();
				matcher.declare_capture(capture_name, vec![matches]).and(Ok(true))
			} else {
				submatcher.unmatch(parser);
				Ok(false)
			}
		},

		other => 
			if let Some(groups) = parser.get_groups(other) {
				drop(submatcher);
				let mut subpattern = matcher.subpattern();

				let groups = groups.to_vec(); // lol, i wish this were better.

				// `groups` is already sorted by priority
				for syntax in groups {
					if syntax.does_match(&mut subpattern, parser)? {
						let matches = subpattern.finish();
						// matcher.declare_capture(capture_name, vec![matches])?;
						return Ok(true);
					}
				}

				// TODO: should we unmatch the submatches?
				return Ok(false);
			} else {
				Err(parser.error(format!("unknown capture type {}", other).into()))
			},
	}
}

impl<'a> PatternAtom<'a> {
	fn does_match(
		&self,
		matcher: &mut Matcher<'a, '_, '_>,
		parser: &mut Parser<'a>
	) -> Result<'a, bool> {
		match self {
			Self::Capture(capture_name, PatternKind::Name(name)) => {
				does_match_named(capture_name, name, matcher, parser)
			},
			Self::Capture(capture_name, PatternKind::Body(ParenType::Round, body)) => {
				let mut submatcher = matcher.submatcher();
				if body.does_match(&mut submatcher, parser)? {
					let matches = submatcher.finish();
					matcher.declare_capture(capture_name, vec![matches]).and(Ok(true))
				} else {
					submatcher.unmatch(parser);
					Ok(false)
				}
			},
			Self::Capture(_, _) => todo!(),
			Self::Paren(_, _) => todo!(),
			Self::Token(token) => {
				// TODO: we should allow macros to match before we currently match.
				// but that requires a way for us to keep track of what's been matched yet.
				if let Some(token) = parser.take_if_contents_bypass_syntax(token.contents)? {
					matcher.push(token);
					Ok(true)
				} else {
					Ok(false)
				}
			},
		}
	}
}
