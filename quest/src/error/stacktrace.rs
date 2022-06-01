use crate::Result;
use crate::vm::{SourceLocation, Frame};
use std::fmt::{self, Display, Formatter};


/// A single stackframe
#[derive(Debug)]
#[must_use]
pub struct Stackframe {
	source_location: SourceLocation,
	name: Option<String>
}

impl Stackframe {
	/// Creates a new [`Stackframe`].
	pub const fn new(source_location: SourceLocation, name: Option<String>) -> Self {
		Self { source_location, name }
	}

	/// Fetches the source location of the stackframe.
	pub const fn source_location(&self) -> &SourceLocation {
		&self.source_location
	}

	/// Gets the name of the stackframe (which may not exist, for lambda functions).
	// TODO: somehow get this in const context?
	pub fn name(&self) -> Option<&str> {
		self.name.as_deref()
	}
}

impl Display for Stackframe {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{} ({})", self.source_location, self.name.as_deref().unwrap_or("<unnamed>"))
	}
}

/// A Stacktrace in Quest, representing the callstack at a point in time during execution.
#[derive(Debug)]
#[must_use]
pub struct Stacktrace(Vec<Stackframe>);

impl Stacktrace {
	/// Creates a new, empty [`Stacktrace`] without any stackframes.
	///
	/// This is useful for when you want to create an Error, but don't want the performance impact
	/// of generating a stackframe.
	pub const fn empty() -> Self {
		Self(Vec::new())
	}

	/// Creates a [`Stacktrace`] of the current stackframe.
	pub fn new() -> Result<Self> {
		Frame::with_stackframes(|frames| {
			let mut locations = Vec::with_capacity(frames.len().saturating_sub(1));

			// We skip the first one, as it's the "global frame," which doesn't have a location.
			for frame in frames.iter().skip(1) {
				let block = frame.as_ref()?.block().as_ref()?;

				let source_location = block.source_location().clone();
				let name = if let Some(name) = block.name()? {
					Some(name.as_ref()?.to_string())
				} else {
					None
				};

				locations.push(Stackframe::new(source_location, name));
			}

			Ok(Self(locations))
		})
	}

	/// Gets the list of [`Stackframe`s].
	pub fn stackframes(&self) -> &[Stackframe] {
		// todo: get this in const context?
		&*self.0
	}
}

impl Display for Stacktrace {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.0.is_empty() {
			write!(f, "<no stacktrace provided>")?;
			return Ok(());
		}

		for (i, stackframe) in self.0.iter().enumerate() {
			writeln!(f, "#{} {}", i + 1, stackframe)?;
		}

		Ok(())
	}
}
