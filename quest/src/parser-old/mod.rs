mod ast;
mod error;
mod stream;
mod token;
mod plugin;

pub use plugin::Plugin;
pub use ast::Ast;
pub use error::{Error, ErrorKind, Result};
pub use stream::Stream;
pub use token::{SpannedToken, Token};

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
	_stream: Stream<'a>,
}
