use super::{SourceLocation, Span};

#[derive(Debug)]
pub struct Stream<'a> {
	filename: Option<&'a str>,
	src: &'a str,
	lineno: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SpanStart<'a> {
	line_start: &'a str,
	lineno: usize,
}

impl<'a> SpanStart<'a> {
	pub fn finish(self, stream: &mut Stream<'a>) -> Span<'a> {
		let _ = self.line_start;

		Span {
			filename: stream.filename,
			lines: "", // todo,
			lines_start: self.lineno,
			lines_end: stream.lineno,
		}
	}
}

impl<'a> Stream<'a> {
	pub fn new(src: &'a str, filename: Option<&'a str>) -> Self {
		Self {
			filename,
			src,
			lineno: 1,
		}
	}

	pub const fn filename(&self) -> Option<&'a str> {
		self.filename
	}

	pub const fn src(&self) -> &'a str {
		self.src
	}

	pub const fn lineno(&self) -> usize {
		self.lineno
	}

	pub const fn is_eof(&self) -> bool {
		self.src.is_empty()
	}

	pub fn span_start(&self) -> SpanStart<'a> {
		SpanStart {
			line_start: "<todo>",
			lineno: self.lineno,
		}
	}

	pub fn source_location(&self) -> SourceLocation<'a> {
		SourceLocation {
			file: self.filename,
			line: "<todo>",
			lineno: self.lineno,
		}
	}

	pub fn error(&self, kind: super::ErrorKind) -> super::Error<'a> {
		super::Error {
			kind,
			src: self.source_location(),
		}
	}

	pub fn peek(&self) -> Option<char> {
		self.src.chars().next()
	}

	pub fn advance(&mut self) {
		let mut c = self.src.chars();
		c.next();
		self.src = c.as_str();
	}

	pub fn next_char(&mut self) -> Option<char> {
		let c = self.peek();
		self.advance();
		c
	}

	pub fn take_if_chr(&mut self, chr: char) -> bool {
		self.take_if(|c| c == chr).is_some()
	}

	pub fn take_str(&mut self, s: &str) -> bool {
		if let Some(src) = self.src.strip_prefix(s) {
			self.src = src;
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
		}

		std::mem::replace(&mut self.src, "")
	}
}
