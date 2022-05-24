use super::{Error, ErrorKind, Result, Stream, Token};
use crate::parse::syntax::{Syntax, MAX_PRIORITY};
use crate::parse::token::TokenContents;
use std::path::Path;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Parser<'a> {
	syntaxes: [Vec<Rc<Syntax<'a>>>; MAX_PRIORITY + 1], // `+1` because `MAX_PRIORITY` is still a valid priority
	groups: HashMap<&'a str, Vec<Rc<Syntax<'a>>>>,
	stream: Stream<'a>,
	peeked_tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
	#[must_use]
	pub fn new(src: &'a str, filename: Option<&'a Path>) -> Self {
		const EMPTY_VEC: Vec<Rc<Syntax<'static>>> = Vec::new();

		Self {
			syntaxes: [EMPTY_VEC; MAX_PRIORITY + 1],
			groups: HashMap::default(),
			stream: Stream::new(src, filename),
			peeked_tokens: Vec::new(),
		}
	}

	pub fn error(&self, kind: ErrorKind) -> Error<'a> {
		self.stream.error(kind.into())
	}

	#[must_use]
	pub fn stream(&self) -> &Stream<'a> {
		&self.stream
	}

	// TODO: this doens't take into account optional order of operations _or_ when it was declared.
	pub fn add_syntax(&mut self, syntax: Syntax<'a>) {
		let syntax = Rc::new(syntax);

		if let Some(group) = syntax.group() {
			self.groups.entry(group).or_default().push(syntax.clone());
		}

		self.syntaxes[MAX_PRIORITY - syntax.priority()].insert(0, syntax);
	}

	pub fn get_group(&self, name: &str) -> Option<&[Rc<Syntax<'a>>]> {
		self.groups.get(name).map(Vec::as_slice)
	}

	#[must_use]
	pub fn location(&self) -> super::SourceLocation<'a> {
		self.stream.location()
	}

	pub fn untake(&mut self, token: Token<'a>) {
		self.peeked_tokens.push(token);
	}

	pub fn untake_tokens<I>(&mut self, tokens: I)
	where
		I: IntoIterator<Item = Token<'a>>,
		I::IntoIter: DoubleEndedIterator,
	{
		self.peeked_tokens.extend(tokens.into_iter().rev());
	}

	pub fn take(&mut self) -> Result<'a, Option<Token<'a>>> {
		self.expand_syntax()?;
		self.take_bypass_syntax()
	}

	pub fn take_bypass_syntax(&mut self) -> Result<'a, Option<Token<'a>>> {
		if let Some(token) = self.peeked_tokens.pop() {
			Ok(Some(token))
		} else {
			Token::parse(&mut self.stream)
		}
	}

	pub fn is_eof(&mut self) -> Result<'a, bool> {
		Ok(self.peek()?.is_none())
	}

	fn expand_syntax(&mut self) -> Result<'a, ()> {
		for i in 0..self.syntaxes.len() {
			for j in 0..self.syntaxes[i].len() {
				if self.syntaxes[i][j].clone().replace(self)? {
					return self.expand_syntax();
				}
			}
		}

		if let Some(syntax) = Syntax::parse(self)? {
			self.add_syntax(syntax);
			return self.expand_syntax();
		}

		Ok(())
	}

	pub fn peek(&mut self) -> Result<'a, Option<Token<'a>>> {
		self.expand_syntax()?;
		self.peek_bypass_syntax()
	}

	pub fn peek_bypass_syntax(&mut self) -> Result<'a, Option<Token<'a>>> {
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

	pub fn take_if_bypass_syntax(
		&mut self,
		cond: impl FnOnce(Token<'a>) -> bool,
	) -> Result<'a, Option<Token<'a>>> {
		if self.peek_bypass_syntax()?.map_or(false, cond) {
			self.take_bypass_syntax()
		} else {
			Ok(None)
		}
	}

	pub fn take_if_contents_bypass_syntax(
		&mut self,
		contents: TokenContents<'a>,
	) -> Result<'a, Option<Token<'a>>> {
		self.take_if_bypass_syntax(|token| token.contents == contents)
	}
}
