use super::{Compile, Group};
use crate::parse::token::{ParenType, TokenContents};
use crate::parse::{Parser, Result};
use crate::value::ToAny;
use crate::vm::block::{Builder, Local};

#[derive(Debug)]
pub struct Block<'a> {
	args: Option<BlockArgs<'a>>,
	body: Group<'a>,
}

impl<'a> Block<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		let args = BlockArgs::parse(parser)?;

		if let Some(body) = Group::parse(parser, ParenType::Curly)? {
			Ok(Some(Self { args, body }))
		} else if args.is_some() {
			panic!("todo: error because block args were given without a block");
		} else {
			Ok(None)
		}
	}
}

#[derive(Debug)]
struct BlockArgs<'a> {
	args: Vec<&'a str>,
}

impl<'a> BlockArgs<'a> {
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if let Some(token) =
			parser.take_if(|token| matches!(token.contents, TokenContents::Identifier(_)))?
		{
			let ident = match token.contents {
				TokenContents::Identifier(ident) => ident,
				_ => unreachable!(),
			};

			if parser
				.take_if_contents(TokenContents::Symbol("->"))?
				.is_some()
			{
				return Ok(Some(Self { args: vec![ident] }));
			}

			parser.add_back(token);
			return Ok(None);
		}

		let left_paren = if let Some(token) =
			parser.take_if_contents(TokenContents::LeftParen(ParenType::Round))?
		{
			token
		} else {
			return Ok(None);
		};

		let mut arg_tokens = Vec::new();

		while let Some(arg) =
			parser.take_if(|token| matches!(token.contents, TokenContents::Identifier(_)))?
		{
			arg_tokens.push(arg);

			if let Some(comma) = parser.take_if_contents(TokenContents::Comma)? {
				arg_tokens.push(comma);
			} else {
				break;
			}
		}

		if let Some(token) = parser.take_if_contents(TokenContents::RightParen(ParenType::Round))? {
			if parser
				.take_if_contents(TokenContents::Symbol("->"))?
				.is_some()
			{
				let mut args = Vec::with_capacity(arg_tokens.len());
				for token in arg_tokens {
					if let TokenContents::Identifier(name) = token.contents {
						args.push(name);
					}
				}

				return Ok(Some(Self { args }));
			}

			parser.add_back(token);
		}

		for token in arg_tokens.into_iter().rev() {
			parser.add_back(token);
		}

		parser.add_back(left_paren);
		Ok(None)
	}
}

impl Compile for Block<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		let location = crate::vm::SourceLocation::from(self.body.start);

		let mut inner_builder = Builder::new(location, None);
		let scratch = Local::Scratch;

		if let Some(args) = &self.args {
			for arg in &args.args {
				let _ = inner_builder.named_local(arg);
			}
		}

		let span = debug_span!(target: "block_builder", "new block", src=?crate::vm::SourceLocation::from(self.body.start));
		span.in_scope(|| self.body.compile(&mut inner_builder, scratch));
		let block = inner_builder.build();
		builder.constant(block.to_any(), dst);
	}
}
