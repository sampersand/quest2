use crate::parse::token::{Token, TokenContents};
use crate::parse::{Parser, Result};

mod matches;
mod pattern;
mod replacement;

use matches::{Matcher, Matches};
use pattern::Pattern;
use replacement::Replacement;

pub type Priority = usize;
pub const MIN_PRIORITY: Priority = 100;
pub const DEFAULT_PRIORITY: Priority = 75; // it's not common to want to be less than default.

#[derive(Debug)]
pub struct Syntax<'a> {
	group: Option<&'a str>,
	priority: Priority,
	nomatch: bool, // dont use it when matching normal expressions, ie only when used in groups
	pattern: Pattern<'a>,
	replacement: Replacement<'a>,
}

impl<'a> Syntax<'a> {
	pub fn group(&self) -> Option<&'a str> {
		self.group
	}

	pub fn priority(&self) -> Priority {
		self.priority
	}

	pub fn nomatch(&self) -> bool {
		self.nomatch
	}

	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		match parser.take_bypass_syntax()? {
			Some(Token { contents: TokenContents::SyntaxIdentifier(0, "syntax"), .. }) => {}
			Some(token) => {
				parser.untake(token);
				return Ok(None);
			}
			None => return Ok(None),
		}

		let nomatch = parser.take_if_contents_bypass_syntax(TokenContents::Symbol("!"))?.is_some();

		let group = match parser.take()? {
			Some(Token { contents: TokenContents::Identifier(name), .. }) => Some(name),
			Some(token) => {
				parser.untake(token);
				None
			}
			None => None,
		};

		let priority = match parser.take()? {
			Some(Token { contents: TokenContents::Integer(num), .. }) => {
				if num.get() <= MIN_PRIORITY as _ {
					num.get() as Priority
				} else {
					return Err(parser.error(format!("priority must be 0..{MIN_PRIORITY}").into()));
				}
			}
			Some(token) => {
				parser.untake(token);
				DEFAULT_PRIORITY
			}
			None => DEFAULT_PRIORITY,
		};

		let pattern = Pattern::parse(parser)?
			.ok_or_else(|| parser.error("expected pattern for `$syntax`".to_string().into()))?;

		if parser.take_if_contents(TokenContents::Symbol("="))?.is_none() {
			return Err(parser.error("expected `=` after `$syntax` pattern".to_string().into()));
		}

		let replacement = Replacement::parse(parser)?
			.ok_or_else(|| parser.error("expected replacement for `$syntax`".to_string().into()))?;

		if parser.take_if_contents(TokenContents::Semicolon)?.is_none() {
			return Err(parser.error("expected `;` after `$syntax` replacement".to_string().into()));
		}

		Ok(Some(Self { group, priority, nomatch, pattern, replacement }))
	}

	// fn matches(&self, matches: &mut Matches<'a>, parser: &mut Parser<'a>) -> Result<'a, bool> {
	// 	let mut matched_tokens = Vec::new();
	// 	let matches = Matches::new
	// 	Ok(true)
	// }

	fn does_match(
		&self,
		matcher: &mut Matcher<'a, '_, '_>,
		parser: &mut Parser<'a>,
	) -> Result<'a, bool> {
		self.pattern.does_match(matcher, parser)
	}

	pub fn replace(&self, parser: &mut Parser<'a>) -> Result<'a, bool> {
		let mut matched_tokens = Vec::new();
		let mut matches = Matcher::new(&mut matched_tokens);
		if self.does_match(&mut matches, parser)? {
			self.replacement.replace(matches.finish(), parser)?;
			Ok(true)
		} else {
			Ok(false)
		}
	}
}
