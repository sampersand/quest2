use crate::parser::{Parser, Result};
use crate::vm::block::{Local, Builder};
// use crate::parser::token::ParenType;
use super::{Expression, Atom, FnArgs, Primary, AttrAccessKind, Compile};

#[derive(Debug)]
pub enum Assignment<'a> {
	Normal(&'a str, Expression<'a>),
	AttrAccess(Primary<'a>, AttrAccessKind, Atom<'a>, Expression<'a>),
	Index(Primary<'a>, FnArgs<'a>, Expression<'a>),
}

impl<'a> Assignment<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let _ = parser;
		panic!();
	}
}

impl Compile for Assignment<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		let _ = (builder, dst);

		match self {
			_ => todo!()
		}
	}
}
