use crate::vm::block::{Local, Builder};
use crate::parser::{Parser, Result};
use super::{Primary, Assignment, Compile};

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
				let op_local = builder.unnamed_local();
				// builder.constant(Text::from(op).as_any(), op_local);
				lhs.compile(builder, lhs_local);
				builder.str_constant(op, op_local);
				rhs.compile(builder, dst);
				builder.call_attr_simple(lhs_local, op_local, &[dst], dst);
			}
		}
	}
}

