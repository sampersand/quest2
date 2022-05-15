use crate::parser::pattern::{Context, Expandable, Pattern};
use crate::parser::{Parser, Result};
use std::rc::Rc;

#[derive(Debug)]
pub struct Repeat<'a> {
	min: usize,
	max: Option<usize>,
	pattern: Rc<dyn Pattern<'a>>,
}

impl<'a> Repeat<'a> {
	pub fn new(min: usize, max: Option<usize>, pattern: Rc<dyn Pattern<'a>>) -> Option<Self> {
		match max {
			Some(0) => None,
			Some(x) if x < min => None,
			_ => Some(Self { min, max, pattern }),
		}
	}
}

#[derive(Debug)]
pub struct RepeatMatches<'a>(Vec<Box<dyn Expandable<'a> + 'a>>);

impl<'a> Pattern<'a> for Repeat<'a> {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		let mut matches;
		if let Some(pattern_match) = self.pattern.try_match(parser)? {
			matches = Vec::with_capacity(2); // todo: allocate with max-min lor somethin
			matches.push(pattern_match);
		} else if self.min == 0 {
			return Ok(Some(Box::new(RepeatMatches(vec![]))));
		} else {
			return Ok(None);
		}

		if self.max.map_or(true, |x| x != 1) {
			while matches.len() < self.min {
				if let Some(pattern_match) = self.pattern.try_match(parser)? {
					matches.push(pattern_match);
				} else {
					RepeatMatches(matches).deconstruct(parser);
					return Ok(None);
				}
			}

			while self.max.map_or(true, |max| matches.len() < max) {
				if let Some(pattern_match) = self.pattern.try_match(parser)? {
					matches.push(pattern_match);
				} else {
					break;
				}
			}
		}

		Ok(Some(Box::new(RepeatMatches(matches))))
	}
}

impl<'a> Expandable<'a> for RepeatMatches<'a> {
	// TODO: should these two be swapped for which does rev?
	fn expand(&self, parser: &mut Parser<'a>, context: Context) {
		for pattern_match in self.0.iter().rev() {
			pattern_match.expand(parser, context.clone());
		}
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		for pattern_match in self.0.iter().rev() {
			pattern_match.deconstruct(parser);
		}
	}
}
