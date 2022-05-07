use super::{Error, ErrorKind, Pattern, Result, Stream, /*Plugin,*/ Token};
use crate::parser::token::TokenContents;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

#[derive(Debug)]
pub struct Parser<'a> {
	// plugins: Vec<Box<u8>>,
	patterns: HashMap<String, Rc<dyn Pattern<'a>>>,
	stream: Stream<'a>,
	peeked_tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
	pub fn new(src: &'a str, filename: Option<&'a Path>) -> Self {
		Self {
			patterns: HashMap::new(),
			// plugins: vec![],
			stream: Stream::new(src, filename),
			peeked_tokens: vec![],
		}
	}

	pub fn error(&self, kind: ErrorKind) -> Error<'a> {
		self.stream.error(kind)
	}

	// TODO: this doens't take into account optional order of operations _or_ when it was declared.
	pub fn add_pattern(&mut self, name: String, pattern: Rc<dyn Pattern<'a>>) {
		self.patterns.insert(name, pattern);
	}

	pub fn get_pattern(&self, name: &str) -> Option<Rc<dyn Pattern<'a>>> {
		self.patterns.get(name).cloned()
	}

	// pub fn plugins(&self) -> &[Box<u8>] {
	// 	&self.plugins
	// }

	pub fn stream(&self) -> &Stream<'a> {
		&self.stream
	}

	pub fn location(&self) -> super::SourceLocation<'a> {
		self.stream.location()
	}

	pub fn add_back(&mut self, token: Token<'a>) {
		self.peeked_tokens.push(token);
	}

	pub fn take(&mut self) -> Result<'a, Option<Token<'a>>> {
		self.advance()
	}

	pub fn advance(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(token) = self.peeked_tokens.pop() {
			Ok(Some(token))
		} else {
			Token::parse(&mut self.stream)
		}
	}

	pub fn is_eof(&mut self) -> Result<'a, bool> {
		Ok(self.peek()?.is_none())
	}

	pub fn peek(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(&peeked_token) = self.peeked_tokens.last() {
			Ok(Some(peeked_token))
		} else if let Some(token) = Token::parse(&mut self.stream)? {
			self.peeked_tokens.push(token);
			Ok(Some(token))
		} else {
			Ok(None)
		}
	}

	// pub fn take_if_symbol(&mut self, sym: &'a str) -> Result<'a, bool> {
	// 	Ok(self.take_if_contents(TokenContents::Symbol(sym))?.is_some())
	// }

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
			self.advance()
		} else {
			Ok(None)
		}
	}
}
