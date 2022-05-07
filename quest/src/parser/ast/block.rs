use crate::value::AsAny;
use crate::parser::{Parser, Result};
use crate::parser::token::ParenType;
use crate::vm::block::{Local, Builder};
use super::{Group, Compile};

#[derive(Debug)]
pub struct Block<'a> {
	args: BlockArgs<'a>,
	body: Group<'a>,
}

impl<'a> Block<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if let Some(block) = Group::parse(parser, ParenType::Curly)? {
			// todo: arguments to block
			Ok(Some(Self { args: BlockArgs { _todo: &() }, body: block }))
		} else {
			Ok(None)
		}
	}
}

#[derive(Debug)]
struct BlockArgs<'a> {
	_todo: &'a (), // todo
}

impl<'a> BlockArgs<'a> {
	#[allow(unused)]
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let _ = parser;
		panic!();
	}
}

impl Compile for Block<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		// todo: somehow have `builder` have a partially-initialized reference to its stackframe.
		let mut inner_builder = Builder::new(/*self.body.location*/crate::vm::SourceLocation{}, None);
		let scratch = inner_builder.scratch();

		self.body.compile(&mut inner_builder, scratch);
		let frame = inner_builder.build();
		builder.constant(frame.as_any(), dst);
	}
}
