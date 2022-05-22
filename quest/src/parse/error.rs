use super::SourceLocation;

#[derive(Debug)]
#[must_use]
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
	UnknownSyntaxPattern(String),
	UnterminatedGroup,
	Message(String),
}

impl From<String> for ErrorKind {
	fn from(inp: String) -> Self {
		Self::Message(inp)
	}
}
