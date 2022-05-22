use crate::parse::token::{Token, TokenContents};
use crate::parse::{Parser, Result};

mod pattern;
mod replacement;

use pattern::Pattern;
use replacement::Replacement;

pub type Priority = usize;
pub const MAX_PRIORITY: Priority = 100;
pub const DEFAULT_PRIORITY: Priority = MAX_PRIORITY / 2;

#[derive(Debug)]
pub struct Macro<'a> {
	group: Option<&'a str>,
	priority: Priority,
	pattern: Pattern<'a>,
	replacement: Replacement<'a>,
}

impl<'a> Macro<'a> {
	pub fn group(&self) -> Option<&'a str> {
		self.group
	}

	pub fn priority(&self) -> Priority {
		self.priority
	}

	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		match parser.take_bypass_macros()? {
			Some(Token { contents: TokenContents::MacroIdentifier(0, "syntax"), .. }) => {},
			Some(token) => {
				parser.untake(token);
				return Ok(None)
			},
			None => return Ok(None)
		}

		let group = match parser.take()? {
			Some(Token {
				contents: TokenContents::Identifier(name),
				..
			}) => Some(name),
			Some(token) => {
				parser.untake(token);
				None
			},
			None => None,
		};

		let priority = match parser.take()? {
			Some(Token {
				contents: TokenContents::Integer(num),
				..
			}) => if num <= MAX_PRIORITY as _ {
				num as Priority
			} else {
				return Err(parser.error(format!("priority must be 0..{}", MAX_PRIORITY).into()))
			},
			Some(token) => {
				parser.untake(token);
				DEFAULT_PRIORITY
			},
			None => DEFAULT_PRIORITY,
		};

		let pattern = Pattern::parse(parser)?.ok_or_else(|| parser.error("expected pattern for `$syntax`".to_string().into()))?;

		if parser.take_if_contents(TokenContents::Symbol("="))?.is_none() {
			return Err(parser.error("expected `=` after `$syntax` pattern".to_string().into()));
		}

		let replacement = Replacement::parse(parser)?.ok_or_else(|| parser.error("expected replacement for `$syntax`".to_string().into()))?;

		if parser.take_if_contents(TokenContents::Semicolon)?.is_none() {
			return Err(parser.error("expected `;` after `$syntax` replacement".to_string().into()));
		}

		Ok(Some(Self { group, priority, pattern, replacement }))
	}

	pub fn replace(&self, parser: &mut Parser<'a>) -> Result<'a, bool> {
		if let Some(matches) = self.pattern.matches(parser)? {
			self.replacement.replace(matches, parser)?;
			Ok(true)
		} else {
			Ok(false)
		}
	}
}

