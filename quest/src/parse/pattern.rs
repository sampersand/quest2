// mod block_literal;
// mod expression;
mod anytoken;
mod block;
mod capture;
mod exact;
mod identifier;
mod literal;
mod named_pattern;
mod oneof;
mod optional;
mod repeat;
mod sequence;
mod symbol;

pub use anytoken::AnyToken;
pub use block::Block;
pub use capture::Capture;
pub use exact::Exact;
pub use identifier::Identifier;
pub use literal::Literal;
pub use named_pattern::NamedPattern;
pub use oneof::OneOf;
pub use optional::Optional;
pub use repeat::Repeat;
pub use sequence::Sequence;
pub use symbol::Symbol;

use crate::parse::{Parser, Result};

pub trait Pattern<'a>: std::fmt::Debug {
	fn try_match(&self, parser: &mut Parser<'a>)
		-> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>>;
}

// todo: have context contain stuff like matched variables and current nesting depth.
#[derive(Debug, Clone)]
pub struct Context;

pub trait Expandable<'a>: std::fmt::Debug {
	fn expand(&self, parser: &mut Parser<'a>, context: Context);
	fn deconstruct(&self, parser: &mut Parser<'a>);
}

impl<'a> Expandable<'a> for crate::parse::Token<'a> {
	fn expand(&self, parser: &mut Parser<'a>, _: Context) {
		parser.untake(*self);
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		parser.untake(*self);
	}
}
