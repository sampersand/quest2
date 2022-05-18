use crate::parse::pattern::{Context, Expandable, Pattern};
use crate::parse::token::{ParenType, TokenContents};
use crate::parse::{ErrorKind, Parser, Result};

#[derive(Debug)]
pub struct Block;

#[derive(Debug)]
pub struct BlockMatch<'a>(Vec<Box<dyn Expandable<'a> + 'a>>);

impl<'a> Pattern<'a> for Block {
	fn try_match(
		&self,
		parser: &mut Parser<'a>,
	) -> Result<'a, Option<Box<dyn Expandable<'a> + 'a>>> {
		let span = if let Some(token) =
			parser.take_if_contents(TokenContents::LeftParen(ParenType::Curly))?
		{
			token.span
		} else {
			return Ok(None);
		};

		let expressions = Vec::new();

		while parser
			.take_if_contents(TokenContents::LeftParen(ParenType::Curly))?
			.is_none()
		{
			if parser.is_eof()? {
				return Err(span.start.error(ErrorKind::UnterminatedGroup));
			}

			// TODO
		}

		Ok(Some(Box::new(BlockMatch(expressions))))
	}
}

impl<'a> Expandable<'a> for BlockMatch<'a> {
	// TODO: should these two be swapped for which does rev?
	fn expand(&self, parser: &mut Parser<'a>, context: Context) {
		let _ = (parser, context);
		todo!();
	}

	fn deconstruct(&self, parser: &mut Parser<'a>) {
		let _ = parser;
		todo!();
	}
}
