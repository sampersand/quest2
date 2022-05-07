use crate::parser::token::TokenContents;
use crate::parser::{Parser, Result};

mod assignment;
mod atom;
mod block;
mod expression;
mod fnargs;
mod group;
mod primary;
pub use assignment::Assignment;
pub use atom::Atom;
pub use block::Block;
pub use expression::Expression;
pub use fnargs::FnArgs;
pub use group::Group;
pub use primary::Primary;

pub trait Compile: std::fmt::Debug {
	fn compile(&self, builder: &mut crate::vm::block::Builder, dst: crate::vm::block::Local);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrAccessKind {
	Period,
	ColonColon,
}

impl AttrAccessKind {
	pub fn parse<'a>(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser.take_if_contents(TokenContents::Period)?.is_some() {
			Ok(Some(Self::Period))
		} else if parser
			.take_if_contents(TokenContents::ColonColon)?
			.is_some()
		{
			Ok(Some(Self::ColonColon))
		} else {
			Ok(None)
		}
	}
}
