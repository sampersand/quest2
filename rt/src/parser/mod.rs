mod token;
mod stream;
mod error;
mod ast;

pub use ast::{Ast, };

pub use error::{Error, ErrorKind, Result};
pub use token::{Token, SpannedToken};
pub use stream::Stream;


#[derive(Debug, Default)]
pub struct SourceLocation<'a> {
	pub file: Option<&'a str>,
	pub line: &'a str,
	pub lineno: usize,
}

#[derive(Debug, Default)]
pub struct Span<'a> {
	pub filename: Option<&'a str>,
	pub lines: &'a str,
	pub lines_start: usize,
	pub lines_end: usize,
}

pub struct Parser<'a> {
	_stream: Stream<'a>
}
