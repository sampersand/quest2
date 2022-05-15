use crate::parser::pattern::{Expandable, Pattern};
use crate::parser::{Parser, Result};
use std::rc::Rc;

#[derive(Debug)]
pub struct OneOf<'a>(pub Vec<Rc<dyn Pattern<'a>>>);

impl<'a> Pattern<'a> for OneOf<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		for pattern in &self.0 {
			if let Some(pattern) = pattern.try_match(parser)? {
				return Ok(Some(pattern));
			}
		}

		Ok(None)
	}
}
