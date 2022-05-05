use crate::parser::token::{Token, TokenContents};
use crate::parser::{Parser, Result, ErrorKind};

#[derive(Debug, Clone)]
pub enum Pattern<'a> {
	AnyToken,
	Exact(TokenContents<'a>),
	Literal,
	Identifier(Option<&'a str>), // no string given means any identifier.
	Symbol(&'a str),

	Capture(&'a str, Box<Pattern<'a>>),
	NamedPattern(&'a str),

	Sequence(Vec<Pattern<'a>>),
	OneOf(Vec<Pattern<'a>>),
	Repeat {
		pat: Box<Pattern<'a>>,
		min: Option<usize>,
		max: Option<usize>,
	},
}

#[derive(Debug, Clone)]
pub enum PatternMatch<'a> {
	SingleToken(Token<'a>),
	Captured(String, Box<PatternMatch<'a>>),
	Sequence(Vec<PatternMatch<'a>>),
}

impl<'a> PatternMatch<'a> {
	fn deconstruct(self, parser: &mut Parser<'a>) {
		match self {
			Self::SingleToken(token) => parser.add_back(token),
			Self::Captured(_, pat) => pat.deconstruct(parser),
			Self::Sequence(patmatches) => {
				for patmatch in patmatches.iter().rev() {
					patmatch.deconstruct(parser);
				}
			}
		}
	}
}

impl<'a> Pattern<'a> {
	pub fn try_match(&self, parser: &mut Parser<'a>) -> Result<'a, Option<PatternMatch<'a>>> {
		match self {
			Self::AnyToken => Ok(parser.advance()?.map(PatternMatch::SingleToken)),
			Self::Exact(contents) => Ok(parser
				.take_if(|tkn| tkn.contents == *contents)?
				.map(PatternMatch::SingleToken)),
			Self::Literal => Ok(parser
				.take_if(|tkn| {
					matches!(
						tkn.contents,
						TokenContents::Text(_)
							| TokenContents::Integer(_)
							| TokenContents::Float(_)
							| TokenContents::Identifier(_)
					)
				})?
				.map(PatternMatch::SingleToken)),
			Self::Identifier(ident_opt) => Ok(parser
				.take_if(|tkn| {
					if let TokenContents::Identifier(ident) = tkn.contents {
						ident_opt.map_or(true, |i| i == ident)
					} else {
						false
					}
				})?
				.map(PatternMatch::SingleToken)),
			Self::Symbol(sym) => Ok(parser
				.take_if(|tkn| matches!(tkn.contents, TokenContents::Symbol(s) if s == *sym))?
				.map(PatternMatch::SingleToken)),

			Self::Capture(name, pat) => if let Some(patmatch) = pat.try_match(parser)? {
				Ok(Some(PatternMatch::Captured(name.to_string(), Box::new(patmatch))))
			} else {
				Ok(None)
			},

			Self::NamedPattern(name) => if let Some(pat) = parser.get_pattern(&name) {
				pat.try_match(parser)
			} else {
				Err(parser.error(ErrorKind::UnknownMacroPattern(name.to_string())))
			},

			Self::Sequence(pats) => {
				let mut patmatches = Vec::with_capacity(pats.len());
				for pat in pats.iter() {
					if let Some(patmatch) = pat.try_match(parser)? {
						patmatches.push(patmatch)
					} else {
						for patmatch in patmatches.into_iter().rev() {
							patmatch.deconstruct(parser);
						}
						return Ok(None);
					}
				}

				Ok(Some(PatternMatch::Sequence(patmatches)))
			},
			Self::OneOf(pats) => {
				for pat in pats.iter() {
					if let Some(patmatch) = pat.try_match(parser)? {
						return Ok(Some(patmatch));
					}
				}

				Ok(None)
			},
			Self::Repeat { pat, min, max } => {
				let mut patmatches = Vec::new();
				'undo: loop {
					while let Some(patmatch) = pat.try_match(parser)? {
						patmatches.push(patmatch);
						if max.map_or(false, |max| max < patmatches.len()) {
							break 'undo;
						}
					}
					if min.map_or(false, |min| min > patmatches.len()) {
						break 'undo;
					}
					return Ok(Some(PatternMatch::Sequence(patmatches)))
				}
				for patmatch in patmatches.into_iter().rev() {
					patmatch.deconstruct(parser);
				}
				Ok(None)
			},
		}
	}
}
