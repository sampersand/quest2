use crate::parser::{Parser, Result};
use crate::parser::token::ParenType;
use super::Group;

#[derive(Debug)]
pub struct Block<'a> {
	pub args: BlockArgs<'a>,
	pub body: Group<'a>,
}

impl<'a> Block<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if let Some(block) = Group::parse(parser, ParenType::Round)? {
			let _ = block;
			todo!();
		} else {
			Ok(None)
		}
	}
}

#[derive(Debug)]
pub struct BlockArgs<'a> {
	_todo: &'a (), // todo
}

impl<'a> BlockArgs<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let _ = parser;
		panic!();
	}
}
