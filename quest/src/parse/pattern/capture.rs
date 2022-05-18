use crate::parse::pattern::{Context, Expandable, Pattern};
use crate::parse::{Parser, Result};
use std::rc::Rc;

#[derive(Debug)]
pub struct Capture<'a>(pub &'a str, pub Rc<dyn Pattern<'a>>);

#[derive(Debug)]
pub struct CapturedPattern<'a>(&'a str, Box<dyn Expandable<'a> + 'a>);

impl<'a> Pattern<'a> for Capture<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		if let Some(pattern_match) = self.1.try_match(parser)? {
			Ok(Some(Box::new(CapturedPattern(self.0, pattern_match))))
		} else {
			Ok(None)
		}
	}
}

impl<'a> Expandable<'a> for CapturedPattern<'a> {
	fn expand(&self, _: &mut Parser<'a>, _: Context) {
		todo!()
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		self.1.deconstruct(parser);
	}
}
