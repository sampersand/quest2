use crate::{value::Intern, AnyValue};
use std::fmt::{self, Display, Formatter};

mod stacktrace;

pub use stacktrace::Stacktrace;

#[derive(Debug)]
pub struct Error {
	stacktrace: Stacktrace,
	kind: ErrorKind
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
	AlreadyLocked(AnyValue),
	ValueFrozen(AnyValue),
	UnknownAttribute(AnyValue, AnyValue),
	MissingPositionalArgument(usize),
	MissingKeywordArgument(&'static str),
	InvalidTypeGiven {
		expected: &'static str,
		given: &'static str,
	},
	ConversionFailed(AnyValue, Intern),
	Message(String),
	Return {
		value: AnyValue,
		from_frame: Option<AnyValue>, // If it's `None`, that means the current frame.
	},
	KeywordsGivenWhenNotExpected,
	PositionalArgumentMismatch { given: usize, expected: usize },
	StackframeIsCurrentlyRunning(AnyValue),
}

impl Error {
	pub fn new(kind: ErrorKind) -> Self {
		Self {
			stacktrace: Stacktrace::new().expect("<unable to fetch stacktrace when making error>"),
			kind
		}
	}

	pub fn new_no_stacktrace(kind: ErrorKind) -> Self {
		Self {
			stacktrace: Stacktrace::empty(),
			kind
		}
	}

	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}

	pub fn stacktrace(&self) -> &Stacktrace {
		&self.stacktrace
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "error: ")?;
		match &self.kind {
			ErrorKind::UnknownAttribute(value, attr) => {
				write!(f, "unknown attribute {attr:?} for {value:?}")?
			},
			ErrorKind::AlreadyLocked(value) => write!(f, "value {value:?} is already locked")?,
			ErrorKind::ValueFrozen(value) => write!(f, "value {value:?} is frozen")?,
			ErrorKind::MissingPositionalArgument(arg) => write!(f, "missing positional argument {arg:?}")?,
			ErrorKind::MissingKeywordArgument(arg) => write!(f, "missing keyword argument {arg:?}")?,
			ErrorKind::ConversionFailed(value, conv) => {
				write!(f, "conversion {value:?} failed for {conv:?}")?
			},
			ErrorKind::InvalidTypeGiven { expected, given } => {
				write!(f, "invalid type {given:?}, expected {expected:?}")?
			},
			ErrorKind::Message(msg) => f.write_str(msg)?,
			ErrorKind::Return { value, from_frame } => {
				write!(f, "returning value {value:?} from frame {from_frame:?}")?
			},
			ErrorKind::KeywordsGivenWhenNotExpected => write!(f, "keyword arguments given when none expected")?,
			ErrorKind::PositionalArgumentMismatch { given, expected } => {
				write!(f, "positional argument count mismatch (given {given} expected {expected})")?
			},
			ErrorKind::StackframeIsCurrentlyRunning(frame) => write!(f, "frame {:?} is currently executing", frame)?,
		}

		write!(f, "\nstacktrace:\n{}", self.stacktrace)
	}
}

impl From<String> for ErrorKind {
	fn from(msg: String) -> Self {
		Self::Message(msg)
	}
}

impl From<String> for Error {
	fn from(msg: String) -> Self {
		ErrorKind::from(msg).into()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Self {
		Self::new(kind)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		None
	}
}
