use super::Plugin;
use crate::parser::{Parser, Result, Token};

#[derive(Debug)]
pub struct Expression<'a> {
	body: Vec<Token<'a>>,
}

impl<'a> Plugin<'a> for Expression<'a> {
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		Ok(None)
	}

	fn compile(self, builder: &mut crate::vm::block::Builder) {
		let _ = builder;
		todo!();
	}
}
