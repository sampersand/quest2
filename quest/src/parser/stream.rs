use crate::parser::{Error, ErrorKind, SourceLocation};
use std::path::Path;

#[derive(Debug)]
pub struct Stream<'a> {
	filename: Option<&'a Path>,
	src: &'a str,
	line: usize,
	column: usize,
}

impl<'a> Stream<'a> {
	pub const fn new(src: &'a str, filename: Option<&'a Path>) -> Self {
		Self {
			src,
			filename,
			line: 1,
			column: 1,
		}
	}

	pub const fn location(&self) -> SourceLocation<'a> {
		SourceLocation {
			filename: self.filename,
			line: self.line,
			column: self.column,
		}
	}

	pub const fn error(&self, kind: ErrorKind) -> Error<'a> {
		self.location().error(kind)
	}

	pub const fn src(&self) -> &'a str {
		self.src
	}

	pub const fn is_eof(&self) -> bool {
		self.src.is_empty()
	}

	pub fn set_eof(&mut self) {
		self.src = "";
	}

	fn next_line(&mut self) {
		self.line += 1;
		self.column = 1;
	}

	pub fn peek(&self) -> Option<char> {
		self.src.chars().next()
	}

	pub fn peek2(&self) -> Option<char> {
		let mut chars = self.src.chars();
		chars.next();
		chars.next()
	}
	
	pub fn peek3(&self) -> Option<char> {
		let mut chars = self.src.chars();
		chars.next();
		chars.next();
		chars.next()
	}

	pub fn advance(&mut self) {
		debug_assert!(!self.is_eof(), ".advance() when eof");

		let mut chars = self.src.chars();

		match chars.next() {
			Some('\n') => self.next_line(),
			Some(other) => self.column += other.len_utf8(),
			None => {},
		}

		self.src = chars.as_str();
	}

	pub fn take(&mut self) -> Option<char> {
		let chr = self.peek();
		if chr.is_some() {
			self.advance();
		}
		chr
	}

	pub fn starts_with(&mut self, s: &str) -> bool {
		self.src.starts_with(s)
	}

	// Note: don't call this with `\n` or it'll
	pub fn take_str(&mut self, s: &str) -> bool {
		assert!(!s.contains('\n'));

		if let Some(src) = self.src.strip_prefix(s) {
			self.src = src;
			self.column += s.len();
			true
		} else {
			false
		}
	}

	pub fn take_if(&mut self, func: impl FnOnce(char) -> bool) -> Option<char> {
		if let Some(chr) = self.peek() {
			if func(chr) {
				self.advance();
				return Some(chr);
			}
		}

		None
	}

	pub fn take_while(&mut self, mut func: impl FnMut(char) -> bool) -> &'a str {
		for (idx, chr) in self.src.char_indices() {
			if !func(chr) {
				let (ret, src) = self.src.split_at(idx);
				self.src = src;
				return ret;
			}

			if chr == '\n' {
				self.next_line();
			} else {
				self.column += chr.len_utf8();
			}
		}

		std::mem::replace(&mut self.src, "")
	}
}
