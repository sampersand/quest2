use crate::Result;
use crate::vm::{SourceLocation, Frame};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Stacktrace(Vec<(SourceLocation, Option<String>)>);

impl Stacktrace {
	pub fn empty() -> Self {
		Self(Vec::new())
	}

	pub fn new() -> Result<Self> {
		Frame::with_stackframes(|frames| {
			let mut locations = Vec::with_capacity(frames.len().saturating_sub(1));

			// we skip the first one, as it's the outermost one
			for frame in frames.iter().skip(1) {
				let block = frame.as_ref()?.block().as_ref()?;

				let source_location = block.source_location().clone();
				let name = if let Some(name) = block.name() {
					Some(name.as_ref()?.to_string())
				} else {
					None
				};

				locations.push((source_location, name));
			}

			Ok(Self(locations))
		})
	}
}

impl Display for Stacktrace {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.0.is_empty() {
			write!(f, "<no stacktrace provided>")?;
			return Ok(());
		}

		for (i, (location, name)) in self.0.iter().enumerate() {
			write!(f, "#{} {}", i + 1, location)?;

			if let Some(name) = name {
				write!(f, " ({})", name)?;
			} else {
				write!(f, " (<unknown>)")?;
			}

			writeln!(f)?;
		}

		Ok(())
	}
}
