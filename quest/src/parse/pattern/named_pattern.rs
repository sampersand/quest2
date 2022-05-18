use crate::parse::pattern::{Expandable, Pattern};
use crate::parse::{ErrorKind, Parser, Result};

#[derive(Debug)]
pub struct NamedPattern<'a>(pub &'a str);

impl<'a> Pattern<'a> for NamedPattern<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		if let Some(pattern) = parser.get_pattern(self.0) {
			pattern.try_match(parser)
		} else {
			Err(parser.error(ErrorKind::UnknownMacroPattern(self.0.to_string())))
		}
	}
}
