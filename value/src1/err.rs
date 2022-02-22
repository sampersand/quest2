use std::fmt::{self, Display, Formatter};
use std::error::Error as ErrorTrait;

use crate::gc::AlreadyLockedError;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	AlreadyLocked(AlreadyLockedError)
}

pub type Result<T> = std::result::Result::<T, Error>;

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::AlreadyLocked(err) => Display::fmt(err, f),
		}
	}
}

impl ErrorTrait for Error {
	fn cause(&self) -> Option<&(dyn ErrorTrait)> {
		match self {
			Self::AlreadyLocked(err) => Some(err),
			// _ => None
		}
	}
}

impl From<AlreadyLockedError> for Error {
	fn from(err: AlreadyLockedError) -> Self {
		Self::AlreadyLocked(err)
	}
}
