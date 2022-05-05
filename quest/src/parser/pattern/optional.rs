use crate::parser::pattern::{Expandable, Pattern, Context};
use crate::parser::{Parser, Result};
use std::rc::Rc;

#[derive(Debug)]
pub struct Optional<'a>(pub Rc<dyn Pattern<'a>>);

#[derive(Debug)]
pub struct OptionalMatch<'a>(Option<Box<dyn Expandable<'a> + 'a>>);

impl<'a> Pattern<'a> for Optional<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		Ok(Some(Box::new(OptionalMatch(self.0.try_match(parser)?))))
	}
}

impl<'a> Expandable<'a> for OptionalMatch<'a> {
		// TODO: should these two be swapped for which does rev?
	fn expand(&self, parser: &mut Parser<'a>, context: Context) {
		if let Some(opt_match) = &self.0 {
			opt_match.expand(parser, context);
		}
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		if let Some(opt_match) = &self.0 {
			opt_match.deconstruct(parser);
		}
	}
}
