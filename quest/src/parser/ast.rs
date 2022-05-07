use crate::parser::token::{TokenContents};
use crate::parser::{Parser, Result};

mod atom;
mod group;
mod block;
mod primary;
mod expression;
mod assignment;
mod fnargs;
pub use atom::Atom;
pub use group::Group;
pub use block::Block;
pub use primary::Primary;
pub use expression::Expression;
pub use assignment::Assignment;
pub use fnargs::FnArgs;

pub trait Compile : std::fmt::Debug {
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

