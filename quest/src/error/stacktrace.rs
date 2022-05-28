use crate::Result;
use crate::vm::{SourceLocation, Frame};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Stacktrace(Vec<(SourceLocation, Option<String>)>);

impl Stacktrace {
	pub fn new() -> Result<Self> {
		Frame::with_stackframes(|frames| {
			let mut locations = Vec::with_capacity(frames.len());

			for frame in frames {
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
		for (i, (location, name)) in self.0.iter().enumerate() {
			write!(f, "#{} {}", i, location)?;

			if let Some(name) = name {
				write!(f, " ({})", name)?;
			}

			writeln!(f)?;
		}

		Ok(())
	}
}
