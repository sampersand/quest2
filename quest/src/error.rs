use crate::{value::Intern, AnyValue};
use std::fmt::{self, Display, Formatter};
use crate::vm::SourceLocation;


#[derive(Debug)]
pub struct Stacktrace(Vec<SourceLocation>);

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
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

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::UnknownAttribute(value, attr) => {
				write!(f, "unknown attribute {attr:?} for {value:?}")
			},
			Self::AlreadyLocked(value) => write!(f, "value {value:p} is already locked"),
			Self::ValueFrozen(value) => write!(f, "value {value:p} is frozen"),
			Self::MissingPositionalArgument(arg) => write!(f, "missing positional argument {arg:?}"),
			Self::MissingKeywordArgument(arg) => write!(f, "missing keyword argument {arg:?}"),
			Self::ConversionFailed(value, conv) => {
				write!(f, "conversion {value:?} failed for {conv:?}")
			},
			Self::InvalidTypeGiven { expected, given } => {
				write!(f, "invalid type {given:?}, expected {expected:?}")
			},
			Self::Message(msg) => f.write_str(msg),
			Self::Return { value, from_frame } => {
				write!(f, "returning value {value:?} from frame {from_frame:?}")
			},
			Self::KeywordsGivenWhenNotExpected => write!(f, "keyword arguments given when none expected"),
			Self::PositionalArgumentMismatch { given, expected } => {
				write!(f, "positional argument count mismatch (given {given} expected {expected})")
			},
			Self::StackframeIsCurrentlyRunning(frame) => write!(f, "frame {:?} is currently executing", frame),
		}
	}
}

impl From<String> for Error {
	fn from(msg: String) -> Self {
		Self::Message(msg)
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		None
	}
}
