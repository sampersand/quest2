use super::SourceLocation;

#[derive(Debug)]
pub struct Error<'a> {
	pub kind: ErrorKind,
	pub src: SourceLocation<'a>,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(Debug)]
pub enum ErrorKind {
	UnexpectedEOF,
	BadCharacter,
	UnterminatedQuote,
	InvalidEscape,
}
