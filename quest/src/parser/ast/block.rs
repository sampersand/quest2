use crate::value::AsAny;
use crate::parser::{Parser, Result};
use crate::parser::token::{ParenType, TokenContents};
use crate::vm::block::{Local, Builder};
use super::{Group, Compile};

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
	args: Vec<&'a str>
}

impl<'a> BlockArgs<'a> {
	fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if let Some(token) = parser.take_if(|token| matches!(token.contents, TokenContents::Identifier(_)))? {
			let ident = match token.contents {
				TokenContents::Identifier(ident) => ident,
				_ => unreachable!()
			};

			if parser.take_if_contents(TokenContents::Symbol("->"))?.is_some() {
				return Ok(Some(Self { args: vec![ident] }));
			} else {
				parser.add_back(token);
				return Ok(None);
			}
		}

		let left_paren = if let Some(token) = parser.take_if_contents(TokenContents::LeftParen(ParenType::Round))? {
			token
		} else {
			return Ok(None);
		};

		let mut arg_tokens = Vec::new();

		while let Some(arg) = parser.take_if(|token| matches!(token.contents, TokenContents::Identifier(_)))? {
			arg_tokens.push(arg);

			if let Some(comma) = parser.take_if_contents(TokenContents::Comma)? {
				arg_tokens.push(comma);
			} else {
				break;
			}
		}

		let mut right_paren = None;

		'undo: loop {
			if let Some(token) = parser.take_if_contents(TokenContents::RightParen(ParenType::Round))? {
				right_paren = Some(token);
			} else {
				break 'undo;
			}

			if parser.take_if_contents(TokenContents::Symbol("->"))?.is_none() {
				break 'undo;
			}

			let mut args = Vec::with_capacity(arg_tokens.len());
			for token in arg_tokens {
				if let TokenContents::Identifier(name) = token.contents {
					args.push(name)
				}
			}

			return Ok(Some(Self { args }));
		}

		if let Some(rparen) = right_paren {
			parser.add_back(rparen);
		}
		for token in arg_tokens.iter().rev() {
			parser.add_back(*token);
		}
		parser.add_back(left_paren);
		println!("{:?}", parser);
		println!("{:?}", parser.peek()?);
		// println!("{:?}", parser.advance()?);
		// println!("{:?}", parser.advance()?);
		// println!("{:?}", parser.advance()?);
		// println!("{:?}", parser.advance()?);
		Ok(None)
	}
}

impl Compile for Block<'_> {
	fn compile(&self, builder: &mut Builder, dst: Local) {
		// todo: somehow have `builder` have a partially-initialized reference to its stackframe.
		let mut inner_builder = Builder::new(/*self.body.location*/crate::vm::SourceLocation{}, None);
		let scratch = inner_builder.scratch();

		if let Some(args) = &self.args {
			for arg in &args.args {
				let _ = builder.named_local(arg);
			}
		}

		self.body.compile(&mut inner_builder, scratch);
		let frame = inner_builder.build();
		builder.constant(frame.as_any(), dst);
	}
}
