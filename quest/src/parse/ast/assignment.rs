use super::{Atom, AttrAccessKind, Compile, Expression, FnArgs, Primary};
use crate::parse::token::TokenContents;
use crate::parse::{Parser, Result};
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub enum Assignment<'a> {
	Normal(&'a str, Expression<'a>),
	Index(Box<Primary<'a>>, FnArgs<'a>, Expression<'a>),
	AttrAccess(Box<Primary<'a>>, AttrAccessKind, Atom<'a>, Expression<'a>),
	FnCall(Primary<'a>, Expression<'a>),
}

impl<'a> Assignment<'a> {
	pub fn parse(
		primary: Primary<'a>,
		parser: &mut Parser<'a>,
	) -> Result<'a, std::result::Result<Self, Primary<'a>>> {
		let token = if let Some(token) = parser.take_if_contents(TokenContents::Symbol("="))? {
			token
		} else {
			return Ok(Err(primary));
		};

		let rhs = if let Some(expr) = Expression::parse(parser)? {
			expr
		} else {
			parser.untake(token);
			return Ok(Err(primary));
		};

		match primary {
			Primary::Atom(Atom::Identifier(ident)) => Ok(Ok(Self::Normal(ident, rhs))),
			Primary::Index(source, arguments) => Ok(Ok(Self::Index(source, arguments, rhs))),
			Primary::AttrAccess(source, kind, attr) => {
				Ok(Ok(Self::AttrAccess(source, kind, attr, rhs)))
			},
			other => Ok(Ok(Self::FnCall(other, rhs))),
		}
	}
}

impl Compile for Assignment<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		match self {
			Self::Normal(ident, value) => {
				let local = builder.named_local(ident);
				value.compile(builder, local);
				builder.mov(local, dst);
			},
			Self::Index(source, arguments, value) => {
				let source_local = builder.unnamed_local();
				source.compile(builder, source_local);

				let mut argument_locals = Vec::with_capacity(arguments.arguments.len());
				for argument in &arguments.arguments {
					let local = builder.unnamed_local();
					argument_locals.push(local);
					argument.compile(builder, local);
				}
				value.compile(builder, dst);

				builder.index_assign(source_local, &argument_locals, dst, dst);
			},
			// kind is ignored when assigning.
			Self::AttrAccess(source, _kind, attr, value) => {
				let source_local = builder.unnamed_local();
				let field_local = builder.unnamed_local();

				source.compile(builder, source_local);

				match attr {
					Atom::Identifier(field) => builder.str_constant(field, field_local),
					other => other.compile(builder, field_local),
				}
				value.compile(builder, dst);
				builder.set_attr(source_local, field_local, dst, dst);
			},
			Self::FnCall(prim, expr) => {
				let prim_local = builder.unnamed_local();
				let assign_local = builder.unnamed_local();

				prim.compile(builder, prim_local);
				builder.str_constant("=", assign_local);
				expr.compile(builder, dst);
				builder.call_attr_simple(prim_local, assign_local, &[dst], dst);
			},
		}
		// Normal(&'a str, Expression<'a>),
		// Index(Box<Primary<'a>>, FnArgs<'a>, Expression<'a>),
		// AttrAccess(Box<Primary<'a>>, AttrAccessKind, Atom<'a>, Expression<'a>),

		// 	let _ = (builder, dst);

		// 	match self {
		// 		_ => todo!()
		// 	}
	}
}
