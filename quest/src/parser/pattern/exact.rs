use crate::parser::{Parser, Result};
use crate::parser::pattern::{Pattern, Expandable};
use crate::parser::token::TokenContents;
#[derive(Debug)]
pub struct Exact<'a>(TokenContents<'a>);

impl<'a> Pattern<'a> for Exact<'a> {
	fn try_match(&self, parser: &mut Parser<'a>) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
 		Ok(parser.take_if(|token| token.contents == self.0)?.map(|x| Box::new(x) as _))
	}
}
