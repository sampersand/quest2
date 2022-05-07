use crate::parser::pattern::{Expandable, Pattern};
use crate::parser::token::TokenContents;
use crate::parser::{Parser, Result};

#[derive(Debug)]
pub struct Exact<'a>(pub TokenContents<'a>);

impl<'a> Pattern<'a> for Exact<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(parser
			.take_if(|token| token.contents == self.0)?
			.map(|x| Box::new(x) as _))
	}
}