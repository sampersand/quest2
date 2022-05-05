use super::{Expression, Plugin};
use crate::parser::{token::ParenType, Parser, Result, Token, Pattern};

#[derive(Debug)]
pub struct BlockLiteral<'a> {
	statements: Vec<Expression<'a>>,
}

impl<'a> Plugin<'a> for BlockLiteral<'a> {
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if !parser.guard(TokenKind::LeftParen(ParenType::Curly)) {
			return Ok(None);
		}

		Ok(None)
	}

	fn compile(self, builder: &mut crate::vm::block::Builder) {
		let _ = builder;
		todo!();
	}
}
