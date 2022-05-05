use super::SourceLocation;

#[derive(Debug)]
pub struct Error<'a> {
	pub location: SourceLocation<'a>,
	pub kind: ErrorKind,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(Debug)]
pub enum ErrorKind {
	UnexpectedEOF,
	UnknownTokenStart(char),
	UnterminatedQuote,
	InvalidEscape,
	BadCharacterAfterIntegerLiteral(char),
}
