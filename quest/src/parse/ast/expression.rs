use super::{Assignment, Compile, Primary};
use crate::parse::token::TokenContents;
use crate::parse::{Parser, Result};
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub enum Expression<'a> {
	Primary(Primary<'a>),
	Assignment(Box<Assignment<'a>>),
	BinaryOperator(Box<Expression<'a>>, &'a str, Box<Expression<'a>>),
}

impl<'a> Expression<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let primary = if let Some(primary) = Primary::parse(parser)? {
			primary
		} else {
			return Ok(None);
		};

		let primary = match Assignment::parse(primary, parser)? {
			Ok(assignment) => return Ok(Some(Self::Assignment(Box::new(assignment)))),
			Err(primary) => primary,
		};

		if let Some(token) =
			parser.take_if(|token| matches!(token.contents, TokenContents::Symbol(_)))?
		{
			let sym = match token.contents {
				TokenContents::Symbol(sym) => sym,
				_ => unreachable!(),
			};

			if let Some(rhs) = Expression::parse(parser)? {
				return Ok(Some(Self::BinaryOperator(
					Box::new(Self::Primary(primary)),
					sym,
					Box::new(rhs),
				)));
			}

			// todo: should this be an error?
			parser.untake(token);
		}

		Ok(Some(Self::Primary(primary)))

		// TODO: assignment and binary operator
	}
}

impl Compile for Expression<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		match self {
			Self::Primary(primary) => primary.compile(builder, dst),
			Self::Assignment(assign) => assign.compile(builder, dst),
			Self::BinaryOperator(lhs, op, rhs) => {
				let lhs_local = builder.unnamed_local();
				lhs.compile(builder, lhs_local);

				if let Some(opcode) = crate::vm::Opcode::binary_from_symbol(op) {
					rhs.compile(builder, dst);
					unsafe {
						builder.simple_opcode(opcode, dst, &[lhs_local, dst]);
					}
				} else {
					let op_local = builder.unnamed_local();
					builder.str_constant(op, op_local);
					rhs.compile(builder, dst);
					builder.call_attr_simple(lhs_local, op_local, &[dst], dst);
				}
			},
		}
	}
}
