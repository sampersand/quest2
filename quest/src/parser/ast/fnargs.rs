use crate::parser::{Parser, Result, ErrorKind};
use crate::parser::token::{ParenType, TokenContents};
use super::Expression;

#[derive(Debug)]
pub struct FnArgs<'a> {
	pub arguments: Vec<Expression<'a>> // todo: have splat operator
}

impl<'a> FnArgs<'a> {
	pub fn parse(parser: &mut Parser<'a>, end: ParenType) -> Result<'a, Self> {
		let mut arguments = Vec::new();
		let start = parser.location();

		while parser
			.take_if_contents(TokenContents::RightParen(end))?
			.is_none()
		{
			if parser.is_eof()? {
				return Err(
					start.error(ErrorKind::Message(format!("missing closing {:?} paren for fncall", end))),
				);
			}

			if let Some(expr) = Expression::parse(parser)? {
				arguments.push(expr);
			} else {
				let token = parser.peek()?;
				return Err(parser.error(ErrorKind::Message(
					format!("expected expression in {:?} fnargs, got {:?}", end, token))));
			}

			if parser.take_if_contents(TokenContents::Comma)?.is_none() {
				if parser.take_if_contents(TokenContents::RightParen(end))?.is_none() {
					let token = parser.peek()?;
					return Err(parser.error(ErrorKind::Message(format!("expected closing {:?} `,`, not {:?}", end, token))))
				}

				break;
			}
		}

		Ok(Self { arguments })
	}
}
