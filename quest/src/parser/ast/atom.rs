use super::{Compile, Group};
use crate::parser::token::{ParenType, TokenContents};
use crate::parser::{Parser, Result};
use crate::value::ty::{Float, Integer, Text};
use crate::value::{ToAny, Gc};
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub enum Atom<'a> {
	Integer(Integer),
	Float(Float),
	Text(Gc<Text>),
	Identifier(&'a str),
	Stackframe(isize),
	Group(Group<'a>),
}

impl<'a> Atom<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if let Some(group) = Group::parse(parser, ParenType::Round)? {
			return Ok(Some(Self::Group(group)));
		}

		let token = if let Some(token) = parser.take()? {
			token
		} else {
			return Ok(None);
		};

		match token.contents {
			TokenContents::Integer(int) => Ok(Some(Self::Integer(int))),
			TokenContents::Float(float) => Ok(Some(Self::Float(float))),
			TokenContents::Text(text) => Ok(Some(Self::Text(text))),
			TokenContents::Identifier(ident) => Ok(Some(Self::Identifier(ident))),
			TokenContents::Stackframe(depth) => Ok(Some(Self::Stackframe(depth))),
			_ => {
				parser.add_back(token);
				Ok(None)
			},
		}
	}
}

impl Compile for Atom<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		match self {
			Self::Integer(integer) => builder.constant((*integer).to_any(), dst),
			Self::Float(float) => builder.constant((*float).to_any(), dst),
			Self::Text(text) => builder.constant((*text).to_any(), dst),
			Self::Group(group) => group.compile(builder, dst),
			Self::Identifier(identifier) => {
				let local = builder.named_local(identifier);
				builder.mov(local, dst);
			},
			Self::Stackframe(stackframe) => builder.stackframe(*stackframe, dst),
		}
	}
}
