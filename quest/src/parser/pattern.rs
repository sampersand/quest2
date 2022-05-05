// mod block_literal;
// mod expression;
mod anytoken;
mod capture;
mod exact;
mod identifier;
mod literal;
mod named_pattern;
mod oneof;
mod repeat;
mod sequence;
mod symbol;
mod optional;

pub use anytoken::AnyToken;
pub use capture::Capture;
pub use exact::Exact;
pub use identifier::Identifier;
pub use literal::Literal;
pub use named_pattern::NamedPattern;
pub use oneof::OneOf;
pub use repeat::Repeat;
pub use sequence::Sequence;
pub use symbol::Symbol;
pub use optional::Optional;

use crate::parser::{Parser, Result};

pub trait Pattern<'a> : std::fmt::Debug {
	fn try_match(&self, parser: &mut Parser<'a>)
		-> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>>;
}

// todo: have context contain stuff like matched variables and current nesting depth.
#[derive(Debug, Clone)]
pub struct Context;

pub trait Expandable<'a> : std::fmt::Debug {
	fn expand(&self, parser: &mut Parser<'a>, context: Context);
	fn deconstruct(&self, parser: &mut Parser<'a>);
}

impl<'a> Expandable<'a> for crate::parser::Token<'a> {
	fn expand(&self, parser: &mut Parser<'a>, _: Context) {
		parser.add_back(*self);
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		parser.add_back(*self);
	}
}

// 			Self::Sequence(pats) => {
// 				let mut patmatches = Vec::with_capacity(pats.len());
// 				for pat in pats.iter() {
// 					if let Some(patmatch) = pat.try_match(parser)? {
// 						patmatches.push(patmatch)
// 					} else {
// 						for patmatch in patmatches.into_iter().rev() {
// 							patmatch.deconstruct(parser);
// 						}
// 						return Ok(None);
// 					}
// 				}

// 				Ok(Some(PatternMatch::Sequence(patmatches)))
// 			},

// 			Self::Repeat { pat, min, max } => {
// 				let mut patmatches = Vec::new();
// 				'undo: loop {
// 					while let Some(patmatch) = pat.try_match(parser)? {
// 						patmatches.push(patmatch);
// 						if max.map_or(false, |max| max < patmatches.len()) {
// 							break 'undo;
// 						}
// 					}
// 					if min.map_or(false, |min| min > patmatches.len()) {
// 						break 'undo;
// 					}
// 					return Ok(Some(PatternMatch::Sequence(patmatches)))
// 				}
// 				for patmatch in patmatches.into_iter().rev() {
// 					patmatch.deconstruct(parser);
// 				}
// 				Ok(None)
// 			},
// 		}
// 	}
// }
