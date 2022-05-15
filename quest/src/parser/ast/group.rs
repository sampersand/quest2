use super::{Compile, Expression};
use crate::parser::token::{ParenType, TokenContents};
use crate::parser::{ErrorKind, Parser, Result, SourceLocation};
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub struct Group<'a> {
	pub start: SourceLocation<'a>,
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
	pub fn parse_all(parser: &mut Parser<'a>) -> Result<'a, Self> {
		let start = parser.location();
		let mut statements = Vec::new();
		let mut end_in_semicolon = false;

		while !parser.is_eof()? {
			while parser.take_if_contents(TokenContents::Semicolon)?.is_some() {
				// remove leading semicolons
			}

			if let Some(statement) = Statement::parse(parser)? {
				statements.push(statement);
			} else {
				let token = parser.peek()?;
				return Err(
					parser.error(ErrorKind::Message(format!("expected expression got {token:?}"))),
				);
			}

			if parser.take_if_contents(TokenContents::Semicolon)?.is_some() {
				end_in_semicolon = true;
			} else {
				end_in_semicolon = false;
				break;
			}
		}

		if !parser.is_eof()? {
			let token = parser.peek()?;
			return Err(
				parser.error(ErrorKind::Message(format!("unknown token after expr: {token:?}"))),
			);
		}

		Ok(Self {
			start,
			statements,
			end_in_semicolon,
		})
	}

	pub fn parse(parser: &mut Parser<'a>, paren: ParenType) -> Result<'a, Option<Self>> {
		let start = parser.location();

		if parser
			.take_if_contents(TokenContents::LeftParen(paren))?
			.is_none()
		{
			return Ok(None);
		};

		let mut statements = Vec::new();
		let mut end_in_semicolon = false;

		while parser
			.take_if_contents(TokenContents::RightParen(paren))?
			.is_none()
		{
			if parser.is_eof()? {
				return Err(
					start.error(ErrorKind::Message(format!("missing closing {paren:?} paren"))),
				);
			}

			while parser.take_if_contents(TokenContents::Semicolon)?.is_some() {
				// remove leading semicolons
			}

			if let Some(statement) = Statement::parse(parser)? {
				statements.push(statement);
			} else {
				let token = parser.peek()?;
				return Err(parser.error(ErrorKind::Message(format!(
					"expected expression in {paren:?} group, got {token:?}"
				))));
			}

			if parser.take_if_contents(TokenContents::Semicolon)?.is_some() {
				end_in_semicolon = true;
				continue;
			}

			end_in_semicolon = false;
			if parser
				.take_if_contents(TokenContents::RightParen(paren))?
				.is_none()
			{
				let token = parser.peek()?;
				return Err(
					parser.error(ErrorKind::Message(format!("unknown token after expr: {token:?}"))),
				);
			}
			break;
		}

		Ok(Some(Self {
			start,
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

				builder.create_list(&locals, dst);
			},
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
