use crate::value::AnyValue;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	AlreadyLocked(AnyValue)
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::AlreadyLocked(value) => write!(f, "value {:p} is already locked", value),
		}
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		None
	}
}
