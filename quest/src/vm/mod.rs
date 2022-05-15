#![allow(unused)]

mod args;
pub mod block;
pub mod bytecode;
mod frame;
mod stackframe;

pub use args::Args;
pub use block::Block;
pub use bytecode::Opcode;
pub use frame::Frame;
pub use stackframe::Stackframe;

#[derive(Clone)]
pub struct SourceLocation {
	pub file: std::path::PathBuf,
	pub line: usize,
	pub column: usize,
}

impl std::fmt::Debug for SourceLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
	}
}

impl Default for SourceLocation {
	fn default() -> Self {
		Self {
			file: "<unknown>".into(),
			line: 0,
			column: 0,
		}
	}
}

impl From<crate::parser::SourceLocation<'_>> for SourceLocation {
	fn from(inp: crate::parser::SourceLocation<'_>) -> Self {
		Self {
			file: inp
				.filename
				.unwrap_or(std::path::Path::new("<unknown>"))
				.into(),
			line: inp.line,
			column: inp.column,
		}
	}
}
