#![allow(unused)]
use super::{token::ParenType, Token};

#[derive(Debug, Clone)]
pub struct Ast<'a> {
	src: Vec<AstSrc<'a>>,
	repl: Vec<AstRepl<'a>>,
}

#[derive(Debug, Clone)]
pub enum AstRepl<'a> {
	Tkn(Token<'a>),
	Seq(Vec<Self>),
	NamedVariable(&'a str),
	NamedSequence(&'a str, Box<Self>),
}

#[derive(Debug, Clone)]
pub enum AstSrc<'a> {
	Named(&'a str, Box<Self>),

	AnyToken,
	LeftParen(ParenType),
	RightParen(ParenType),

	Literal,
	Block,
	AnyIdentifier,
	Identifier(&'a str),
	Symbol(&'a str),

	Sequential(Vec<Self>),
	AnyOneOf(Vec<Self>),

	Repeat {
		name: &'a str,
		min: usize,
		max: Option<usize>,
		what: Box<Self>,
	},
	Optional(&'a str, Box<Self>),
	ZeroOrMore(&'a str, Box<Self>),
	OneOrMore(&'a str, Box<Self>),
}
impl AstSrc<'_> {
	fn expression() -> Self {
		todo!()
	}
}

impl Ast<'_> {
	pub fn make_if() -> Self {
		use AstRepl::*;
		use AstSrc::*;

		/*
			'if' <$cond:expr> <$ifbody:block>
			{ 'else' 'if' <$elseif_cond:expr> <$elseif_body:block> }
			[ 'else' <$else_body:block> ]
		*/
		Self {
			src: vec![
				Identifier("if"),
				Named("cond", Box::new(AstSrc::expression())),
				Named("ifbody", Box::new(Block)),
				ZeroOrMore(
					"s0",
					Box::new(Sequential(vec![
						Identifier("else"),
						Identifier("if"),
						Named("elseif_cond", Box::new(AstSrc::expression())),
						Named("elseif_body", Box::new(Block)),
					])),
				),
				Optional(
					"s1",
					Box::new(Sequential(vec![Identifier("else"), Named("else_body", Box::new(Block))])),
				),
			],
			repl: vec![
				Tkn(Token::Identifier("Kernel")),
				Tkn(Token::Symbol("::")),
				Tkn(Token::Identifier("if")),
				Tkn(Token::LeftParen(ParenType::Round)),
				NamedVariable("cond"),
				Tkn(Token::Symbol(",")),
				NamedVariable("ifbody"),
				NamedSequence(
					"s0",
					Box::new(Seq(vec![
						Tkn(Token::Symbol(",")),
						NamedVariable("elseif_cond"),
						Tkn(Token::Symbol(",")),
						NamedVariable("elseif_body"),
					])),
				),
				NamedSequence("s1", Box::new(NamedVariable("else_body"))),
				Tkn(Token::RightParen(ParenType::Round)),
			],
		}
	}
}
