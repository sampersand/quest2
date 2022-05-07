use crate::parser::{Parser, Result, ErrorKind};
use crate::parser::token::{TokenContents, ParenType};
use crate::vm::block::{Local, Builder};
use super::{Atom, Block, FnArgs, AttrAccessKind, Compile};

#[derive(Debug)]
pub enum Primary<'a> {
	Atom(Atom<'a>),
	Block(Block<'a>),
	Array(FnArgs<'a>),
	UnaryOp(&'a str, Box<Primary<'a>>),
	FnCall(Box<Primary<'a>>, FnArgs<'a>),
	Index(Box<Primary<'a>>, FnArgs<'a>),
	AttrAccess(Box<Primary<'a>>, AttrAccessKind, Atom<'a>),
}

impl<'a> Primary<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let mut primary = if let Some(atom) = Atom::parse(parser)? {
			Self::Atom(atom)
		} else if let Some(block) = Block::parse(parser)? {
			Self::Block(block)
		} else if parser
			.take_if_contents(TokenContents::LeftParen(ParenType::Square))?
			.is_some()
		{
			Self::Array(FnArgs::parse(parser, ParenType::Square)?)
		} else if let Some(token) =
			parser.take_if(|token| matches!(token.contents, TokenContents::Symbol(_)))?
		{
			let symbol = match token.contents {
				TokenContents::Symbol(sym) => sym,
				_ => unreachable!()
			};

			if let Some(rhs) = Self::parse(parser)? {
				Self::UnaryOp(symbol, Box::new(rhs))
			} else {
				// todo: should this be an error or do we put it back?
				parser.add_back(token);
				return Ok(None);
			}
		} else {
			return Ok(None);
		};

		loop {
			primary = if parser
				.take_if_contents(TokenContents::LeftParen(ParenType::Round))?
				.is_some()
			{
				Self::FnCall(Box::new(primary), FnArgs::parse(parser, ParenType::Round)?)
			} else if parser
				.take_if_contents(TokenContents::LeftParen(ParenType::Square))?
				.is_some()
			{
				Self::Index(Box::new(primary), FnArgs::parse(parser, ParenType::Square)?)
			} else if let Some(access_kind) = AttrAccessKind::parse(parser)? {
				if let Some(atom) = Atom::parse(parser)? {
					Self::AttrAccess(Box::new(primary), access_kind, atom)
				} else {
					return Err(
						parser.error(ErrorKind::Message("expected atom after `.` or `::`".to_string())),
					);
				}
			} else {
				return Ok(Some(primary));
			};
		}
	}
}

impl Compile for Primary<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		match self {
			Self::Atom(atom) => atom.compile(builder, dst),
			Self::Block(block) => todo!("{:?}", block),
			Self::Array(fnargs) => todo!("{:?}", fnargs),
			Self::UnaryOp(op, primary) => {
				if let Some(opcode) = crate::vm::Opcode::unary_from_symbol(op) {
					primary.compile(builder, dst);
					unsafe {
						builder.simple_opcode(opcode, &[dst, dst]);
					}
				} else {
					let op_local = builder.unnamed_local();
					builder.str_constant(op, op_local);
					primary.compile(builder, dst);
					builder.call_attr_simple(dst, op_local, &[], dst)
				}
			},
			Self::FnCall(function, arguments) => todo!("{:?} {:?}", function, arguments),
			Self::Index(source, index) => todo!("{:?} {:?}", source, index),
			Self::AttrAccess(source, kind, attribute) => todo!("{:?} {:?} {:?}", source, kind, attribute),
		}
	}
}
