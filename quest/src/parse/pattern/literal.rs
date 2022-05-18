use crate::parse::pattern::{Expandable, Pattern};
use crate::parse::token::TokenContents;
use crate::parse::{Parser, Result};

#[derive(Debug)]
pub struct Literal;

impl<'a> Pattern<'a> for Literal {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(parser
			.take_if(|token| {
				matches!(
					token.contents,
					TokenContents::Text(_)
						| TokenContents::Integer(_)
						| TokenContents::Float(_)
						| TokenContents::Identifier(_)
				)
			})?
			.map(|x| Box::new(x) as _))
	}
}
