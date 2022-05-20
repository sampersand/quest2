use super::{Error, ErrorKind, Result, Stream, Token};
use crate::parse::macros::{Macro, MAX_PRIORITY};
use crate::parse::token::TokenContents;
use std::path::Path;
use std::rc::Rc;

#[derive(Debug)]
pub struct Parser<'a> {
	macros: [Vec<Rc<Macro<'a>>>; MAX_PRIORITY + 1], // `+1` because `MAX_PRIORITY` is still a valid priority
	stream: Stream<'a>,
	peeked_tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
	#[must_use]
	pub const fn new(src: &'a str, filename: Option<&'a Path>) -> Self {
		const EMPTY_VEC: Vec<Rc<Macro<'static>>> = Vec::new();

		Self {
			macros: [EMPTY_VEC; MAX_PRIORITY + 1],
			stream: Stream::new(src, filename),
			peeked_tokens: Vec::new(),
		}
	}

	pub fn error(&self, kind: ErrorKind) -> Error<'a> {
		self.stream.error(kind.into())
	}

	// TODO: this doens't take into account optional order of operations _or_ when it was declared.
	pub fn add_macro(&mut self, mac: Macro<'a>) {
		self.macros[MAX_PRIORITY - mac.priority()].push(Rc::new(mac));
	}

	#[must_use]
	pub fn stream(&self) -> &Stream<'a> {
		&self.stream
	}

	#[must_use]
	pub fn location(&self) -> super::SourceLocation<'a> {
		self.stream.location()
	}

	pub fn untake(&mut self, token: Token<'a>) {
		self.peeked_tokens.push(token);
	}

	pub fn take(&mut self) -> Result<'a, Option<Token<'a>>> {
		self.expand_macros()?;
		self.take_bypass_macros()
	}

	pub fn take_bypass_macros(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(token) = self.peeked_tokens.pop() {
			Ok(Some(token))
		} else {
			Token::parse(&mut self.stream)
		}
	}

	pub fn is_eof(&mut self) -> Result<'a, bool> {
		Ok(self.peek()?.is_none())
	}

	fn expand_macros(&mut self) -> Result<'a, ()> {
		for i in 0..self.macros.len() {
			for j in 0..self.macros[i].len() {
				if self.macros[i][j].clone().replace(self)? {
					return self.expand_macros();
				}
			}
		}

		if let Some(mac) = Macro::parse(self)? {
			self.add_macro(mac);
			return self.expand_macros();
		}

		Ok(())
	}

	pub fn peek(&mut self) -> Result<'a, Option<Token<'a>>> {
		self.expand_macros()?;
		self.peek_bypass_macros()
	}

	pub fn peek_bypass_macros(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(&peeked_token) = self.peeked_tokens.last() {
			Ok(Some(peeked_token))
		} else if let Some(token) = Token::parse(&mut self.stream)? {
			self.peeked_tokens.push(token);
			Ok(Some(token))
		} else {
			Ok(None)
		}
	}

	pub fn take_if_contents(
		&mut self,
		contents: TokenContents<'a>,
	) -> Result<'a, Option<Token<'a>>> {
		self.take_if(|token| token.contents == contents)
	}

	pub fn take_if(
		&mut self,
		cond: impl FnOnce(Token<'a>) -> bool,
	) -> Result<'a, Option<Token<'a>>> {
		if self.peek()?.map_or(false, cond) {
			self.take()
		} else {
			Ok(None)
		}
	}

	pub fn take_if_bypass_macros(
		&mut self,
		cond: impl FnOnce(Token<'a>) -> bool,
	) -> Result<'a, Option<Token<'a>>> {
		if self.peek_bypass_macros()?.map_or(false, cond) {
			self.take_bypass_macros()
		} else {
			Ok(None)
		}
	}

	pub fn take_if_contents_bypass_macros(
		&mut self,
		contents: TokenContents<'a>,
	) -> Result<'a, Option<Token<'a>>> {
		self.take_if_bypass_macros(|token| token.contents == contents)
	}
}
