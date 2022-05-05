mod error;
mod stream;
pub mod token;

pub use error::{Error, ErrorKind, Result};
pub use stream::Stream;
pub use token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation<'a> {
	pub filename: Option<&'a std::path::Path>,
	pub line: usize,
	pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'a> {
	pub start: SourceLocation<'a>,
	pub end: SourceLocation<'a>,
}

impl<'a> SourceLocation<'a> {
	pub const fn error(self, kind: ErrorKind) -> Error<'a> {
		Error {
			location: self,
			kind,
		}
	}
}
