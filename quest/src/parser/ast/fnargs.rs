use crate::parser::{Parser, Result};
use crate::parser::token::ParenType;
use crate::vm::block::{Local, Builder};
use super::Compile;

#[derive(Debug)]
pub struct FnArgs<'a> {
	_todo: &'a str,
}

impl<'a> FnArgs<'a> {
	pub fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Self> {
		let _ = (parser, end);
		panic!();
	}
}

impl Compile for FnArgs<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		let _ = (builder, dst);

		match self {
			_ => todo!()
		}
	}
}
