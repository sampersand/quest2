use crate::value::Gc;
use crate::vm::{Block, Frame};
use std::fmt::{self, Display, Formatter};

/// A Stacktrace in Quest, representing the callstack at a point in time during execution.
#[derive(Debug)]
#[must_use]
pub struct Stacktrace(Vec<Gc<Block>>);

impl Stacktrace {
	/// Creates a new, empty [`Stacktrace`] without any frames.
	///
	/// This is useful for when you want to create an Error, but don't want the performance impact
	/// of generating a stackframe.
	pub const fn empty() -> Self {
		Self(Vec::new())
	}

	/// Gets the current stackframe list.
	///
	/// As errors disrupt control flow and generally are propagated upwards, creating them is
	/// infrequent. Thus, we mark this cold.
	#[cold]
	pub fn current() -> Self {
		Frame::with_stackframes(|frames| {
			let mut locations = Vec::with_capacity(frames.len().saturating_sub(1));

			// We skip the first one, as it's the "global frame," which doesn't have a location.
			for frame in frames.iter().skip(1) {
				locations
					.push(frame.as_ref_option().expect("<todo: get block without needing ref?").block());
			}

			Self(locations)
		})
	}

	/// Gets the list of [`Gc<Block>`]s.
	pub fn blocks(&self) -> &[Gc<Block>] {
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

		for (i, block) in self.0.iter().enumerate() {
			write!(f, "#{} ", i + 1)?;

			if let Some(blockref) = block.as_ref_option() {
				match blockref.display() {
					Ok(display) => Display::fmt(&display, f)?,
					Err(err) => write!(f, "<error: {err}>")?,
				}
			} else {
				write!(f, "<error: unable to get the frame>")?;
			}

			writeln!(f)?;
		}

		Ok(())
	}
}
