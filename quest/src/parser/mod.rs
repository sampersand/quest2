mod error;
mod parser;
mod pattern;
pub mod plugins;
mod source_location;
mod stream;
pub mod token;

pub use error::{Error, ErrorKind, Result};
pub use parser::Parser;
pub use pattern::Pattern;
pub use plugins::Plugin;
pub use source_location::SourceLocation;
pub use stream::Stream;
pub use token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'a> {
	pub start: SourceLocation<'a>,
	pub end: SourceLocation<'a>,
}
