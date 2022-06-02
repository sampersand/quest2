use super::{Atom, AttrAccessKind, Block, Compile, FnArgs};
use crate::parse::token::{ParenType, TokenContents};
use crate::parse::{ErrorKind, Parser, Result};
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub enum Primary<'a> {
	Atom(Atom<'a>),
	Block(Block<'a>),
	List(FnArgs<'a>),
	UnaryOp(&'a str, Box<Primary<'a>>),
	// TODO: attribute call.
	FnCall(Box<Primary<'a>>, FnArgs<'a>),
	Index(Box<Primary<'a>>, FnArgs<'a>),
	AttrAccess(Box<Primary<'a>>, AttrAccessKind, Atom<'a>),
}

impl<'a> Primary<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let mut primary = if let Some(block) = Block::parse(parser)? {
			Self::Block(block)
		} else if let Some(atom) = Atom::parse(parser)? {
			Self::Atom(atom)
		} else if parser
			.take_if_contents(TokenContents::LeftParen(ParenType::Square))?
			.is_some()
		{
			Self::List(FnArgs::parse(parser, ParenType::Square)?)
		} else if let Some(token) =
			parser.take_if(|token| matches!(token.contents, TokenContents::Symbol(_)))?
		{
			let symbol = match token.contents {
				TokenContents::Symbol(sym) => sym,
				_ => unreachable!(),
			};

			if let Some(rhs) = Self::parse(parser)? {
				Self::UnaryOp(symbol, Box::new(rhs))
			} else {
				// todo: should this be an error or do we put it back?
				parser.untake(token);
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
			Self::Block(block) => block.compile(builder, dst),
			Self::List(elements) => {
				// TODO: instead make a builder for `create_array` so we dont need to make a temp array.
				let mut element_locals = Vec::with_capacity(elements.arguments.len());
				for element in &elements.arguments {
					let local = builder.unnamed_local();
					element.compile(builder, local);
					element_locals.push(local);
				}
				builder.create_list(&element_locals, dst);
			},
			Self::UnaryOp(op, primary) => {
				if let Some(opcode) = crate::vm::Opcode::unary_from_symbol(op) {
					primary.compile(builder, dst);
					unsafe {
						builder.simple_opcode(opcode, dst, &[dst]);
					}
				} else {
					let op_local = builder.unnamed_local();
					builder.str_constant(op, op_local);
					primary.compile(builder, dst);
					builder.call_attr_simple(dst, op_local, &[], dst);
				}
			},
			Self::FnCall(function, arguments) => {
				let function_local = builder.unnamed_local();
				function.compile(builder, function_local);
				let mut argument_locals = Vec::with_capacity(arguments.arguments.len());
				for argument in &arguments.arguments {
					let local = builder.unnamed_local();
					argument.compile(builder, local);
					argument_locals.push(local);
				}
				if argument_locals.len() <= crate::vm::MAX_ARGUMENTS_FOR_SIMPLE_CALL {
					builder.call_simple(function_local, &argument_locals, dst);
				} else {
					todo!();
					// builder.call(/*function_local, &argument_locals, dst*/);
				}
			},
			Self::Index(source, index) => {
				let source_local = builder.unnamed_local();
				source.compile(builder, source_local);

				let mut argument_locals = Vec::with_capacity(index.arguments.len());
				for argument in &index.arguments {
					let local = builder.unnamed_local();
					argument.compile(builder, local);
					argument_locals.push(local);
				}
				builder.index(source_local, &argument_locals, dst);
			},
			Self::AttrAccess(source, kind, attribute) => {
				let local = builder.unnamed_local();
				source.compile(builder, local);

				// don't parse identifiers straight up
				if let Atom::Identifier(ident) = attribute {
					builder.str_constant(ident, dst);
				} else {
					attribute.compile(builder, dst);
				}

				match kind {
					AttrAccessKind::ColonColon => builder.get_unbound_attr(local, dst, dst),
					AttrAccessKind::Period => builder.get_attr(local, dst, dst),
				}
			},
		}
	}
}
