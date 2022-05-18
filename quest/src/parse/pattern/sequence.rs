use crate::parse::pattern::{Context, Expandable, Pattern};
use crate::parse::{Parser, Result};
use std::rc::Rc;

#[derive(Debug)]
pub struct Sequence<'a>(pub Vec<Rc<dyn Pattern<'a>>>);

#[derive(Debug)]
pub struct SequenceMatches<'a>(Vec<Box<dyn Expandable<'a> + 'a>>);

impl<'a> Pattern<'a> for Sequence<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		if self.0.is_empty() {
			return Ok(None);
		}

		// Don't allocate the entire buffer if we're just going to fail on the first element.
		let mut matches;
		if let Some(pattern_match) = self.0[0].try_match(parser)? {
			matches = Vec::with_capacity(self.0.len());
			matches.push(pattern_match);
		} else {
			return Ok(None);
		}

		for pat in &self.0[1..] {
			if let Some(pattern_match) = pat.try_match(parser)? {
				matches.push(pattern_match);
			} else {
				SequenceMatches(matches).deconstruct(parser);
				return Ok(None);
			}
		}

		Ok(Some(Box::new(SequenceMatches(matches))))
	}
}

impl<'a> Expandable<'a> for SequenceMatches<'a> {
	// TODO: should these two be swapped for which does rev?
	fn expand(&self, parser: &mut Parser<'a>, context: Context) {
		for pattern_match in &self.0 {
			pattern_match.expand(parser, context.clone());
		}
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		for pattern_match in self.0.iter().rev() {
			pattern_match.deconstruct(parser);
		}
	}
}
