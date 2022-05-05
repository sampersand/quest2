use crate::parser::pattern::{Expandable, Pattern};
use crate::parser::token::TokenContents;
use crate::parser::{Parser, Result};

#[derive(Debug)]
pub struct Symbol<'a>(pub Option<&'a str>);

impl<'a> Pattern<'a> for Symbol<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(parser
			.take_if(|token| {
				if let TokenContents::Symbol(sym) = token.contents {
					self.0.map_or(true, |s| s == sym)
				} else {
					false
				}
			})?
			.map(|token| Box::new(token) as _))
	}
}
