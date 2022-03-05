use crate::AnyValue;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	AlreadyLocked(AnyValue),
	ValueFrozen(AnyValue),
	MissingPositionalArgument(usize),
	MissingKeywordArgument(&'static str),
	ConversionFailed(AnyValue, &'static str),
	Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::AlreadyLocked(value) => write!(f, "value {:p} is already locked", value),
			Self::ValueFrozen(value) => write!(f, "value {:p} is frozen", value),
			Self::MissingPositionalArgument(arg) => write!(f, "missing positional argument {:?}", arg),
			Self::MissingKeywordArgument(arg) => write!(f, "missing keyword argument {:?}", arg),
			Self::ConversionFailed(value, conv) => write!(f, "conversion {:?} failed for {:?}", conv, value),
			Self::Message(msg) => write!(f, "{}", msg),
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
