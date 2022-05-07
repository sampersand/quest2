use crate::parser::{Parser, Result, ErrorKind};
use crate::parser::token::{TokenContents, ParenType};
use crate::vm::block::{Local, Builder};
use super::{Expression, Compile};

#[derive(Debug)]
pub struct Group<'a> {
	statements: Vec<Statement<'a>>,
	end_in_semicolon: bool,
}

#[derive(Debug)]
enum Statement<'a> {
	Single(Expression<'a>),
	Many(Vec<Expression<'a>>),
}

impl<'a> Statement<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let first = if let Some(expr) = Expression::parse(parser)? {
			expr
		} else {
			return Ok(None);
		};

		if parser.take_if_contents(TokenContents::Comma)?.is_none() {
			return Ok(Some(Self::Single(first)));
		}

		let mut many = vec![first];
		while let Some(expr) = Expression::parse(parser)? {
			many.push(expr);

			if parser.take_if_contents(TokenContents::Comma)?.is_none() {
				break;
			}
		}

		Ok(Some(Self::Many(many)))
	}
}

impl<'a> Group<'a> {
	pub fn parse(parser: &mut Parser<'a>, paren: ParenType) -> Result<'a, Option<Self>> {
		if parser.take_if_contents(TokenContents::LeftParen(paren))?.is_none() {
			return Ok(None);
		};

		let mut statements = Vec::new();
		let start = parser.location();
		let mut end_in_semicolon = true;

		while parser
			.take_if_contents(TokenContents::RightParen(paren))?
			.is_none()
		{
			if parser.is_eof()? {
				return Err(
					start.error(ErrorKind::Message(format!("missing closing {:?} paren", paren))),
				);
			}

			end_in_semicolon = false;
			while parser.take_if_contents(TokenContents::Semicolon)?.is_some() {
				end_in_semicolon = true; // strip leading semicolons
			}

			if let Some(statement) = Statement::parse(parser)? {
				statements.push(statement);
			} else {
				return Err(parser.error(ErrorKind::Message("unexpected token".to_string())));
			}
		}

		Ok(Some(Self {
			statements,
			end_in_semicolon,
		}))
	}
}

impl Compile for Statement<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		match self {
			Self::Single(expr) => expr.compile(builder, dst),
			Self::Many(many) => {
				let mut locals = Vec::with_capacity(many.len());

				for expr in many {
					let local = builder.unnamed_local();
					locals.push(local);
					expr.compile(builder, local);
				}

				builder.create_array(&locals, dst);
			}
		}
	}
}

impl Compile for Group<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		for statement in &self.statements {
			statement.compile(builder, dst);
		}

		if self.end_in_semicolon {
			builder.constant(Default::default(), dst);
		}
	}
}
