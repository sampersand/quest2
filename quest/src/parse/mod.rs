pub mod ast;
mod error;
mod parser;
mod source_location;
mod stream;
pub mod syntax;
pub mod token;

pub use error::{Error, ErrorKind, Result};
pub use parser::Parser;
pub use source_location::SourceLocation;
pub use stream::Stream;
pub use syntax::Syntax;
pub use token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'a> {
	pub start: SourceLocation<'a>,
	pub end: SourceLocation<'a>,
}
