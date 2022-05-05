use crate::parser::{Parser, Result};
use crate::parser::pattern::{Pattern, Expandable};

#[derive(Debug)]
pub struct AnyToken;

impl<'a> Pattern<'a> for AnyToken {
	fn try_match(&self, parser: &mut Parser<'a>) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(parser.advance()?.map(|token| Box::new(token) as _))
	}
}
