use super::{Result, ErrorKind, Error, Stream, /*Plugin,*/ Token, Pattern};
use std::collections::HashMap;
use std::path::Path;

pub struct Parser<'a> {
	// plugins: Vec<Box<u8>>,
	patterns: HashMap<String, Box<dyn Pattern<'a>>>,
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
	pub fn add_pattern(&mut self, name: String, pattern: Box<dyn Pattern<'a>>) {
		self.patterns.insert(name, pattern);
	}

	pub fn get_pattern(&self, name: &str) -> Option<&dyn Pattern<'a>> {
		self.patterns.get(name).map(|x| &**x)
	}

	// pub fn plugins(&self) -> &[Box<u8>] {
	// 	&self.plugins
	// }

	pub fn stream(&self) -> &Stream<'a> {
		&self.stream
	}

	pub fn add_back(&mut self, token: Token<'a>) {
		self.peeked_tokens.push(token);
	}

	pub fn advance(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(token) = self.peeked_tokens.pop() {
			Ok(Some(token))
		} else {
			Token::parse(&mut self.stream)
		}
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
