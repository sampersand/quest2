use crate::parse::pattern::{Expandable, Pattern};
use crate::parse::token::TokenContents;
use crate::parse::{Parser, Result};

#[derive(Debug)]
pub struct Identifier<'a>(pub Option<&'a str>);

impl<'a> Pattern<'a> for Identifier<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(parser
			.take_if(|token| {
				if let TokenContents::Identifier(ident) = token.contents {
					self.0.map_or(true, |i| i == ident)
				} else {
					false
				}
			})?
			.map(|token| Box::new(token) as _))
	}
}
