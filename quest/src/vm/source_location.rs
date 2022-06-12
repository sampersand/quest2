use std::fmt::{self, Debug, Display, Formatter};
use std::path::PathBuf;

/// Represents a location in quest source code.
///
/// This is distinct from [`parse::SourceLocation`](crate::parse::SourceLocation) in that the `parse`
/// one doesn't own its `file`, whereas this one does. This is because the `parse` one simply uses
/// a reference, where this one is expected to last (potentially) forever.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct SourceLocation {
	/// The filename (if any).
	pub file: Option<PathBuf>,
	/// The line number.
	pub line: usize,
	/// The column number.
	pub column: usize,
}

impl Debug for SourceLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(self, f)
	}
}

impl Display for SourceLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if let Some(file) = self.file.as_ref() {
			Display::fmt(&file.display(), f)?;
		} else {
			f.write_str("(unknown)")?;
		}

		write!(f, ":{}:{}", self.line, self.column)
	}
}

impl From<crate::parse::SourceLocation<'_>> for SourceLocation {
	fn from(inp: crate::parse::SourceLocation<'_>) -> Self {
		Self { file: inp.filename.map(PathBuf::from), line: inp.line, column: inp.column }
	}
}
