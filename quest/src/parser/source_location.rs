use super::{Error, ErrorKind};
use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation<'a> {
	pub filename: Option<&'a Path>,
	pub line: usize,
	pub column: usize,
}

impl<'a> SourceLocation<'a> {
	pub const fn error(self, kind: ErrorKind) -> Error<'a> {
		Error {
			location: self,
			kind,
		}
	}
}

impl Display for SourceLocation<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(filename) = self.filename {
			write!(f, "{}:{}:{}", filename.display(), self.line, self.column)
		} else {
			write!(f, "-e:{}:{}", self.line, self.column)
		}
	}
}
