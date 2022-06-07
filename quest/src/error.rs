use crate::Value;
use std::fmt::{self, Display, Formatter};

mod stacktrace;
pub use stacktrace::Stacktrace;

/// An error type that contains both a [`Stacktrace`] and an [`ErrorKind`].
#[derive(Debug)]
#[must_use]
pub struct Error {
	pub stacktrace: Stacktrace,
	pub kind: ErrorKind,
}

/// Type alias for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Possible errors that can be thrown during execution within Quest.
#[derive(Debug)]
#[must_use]
#[non_exhaustive]
pub enum ErrorKind {
	/// The value's rwlock is already acquired.
	///
	/// This means either [`Gc::as_ref`](crate::value::Gc::as_ref) was called while the value is
	/// mutably borrowed, or [`Gc::as_mut`](crate::value::Gc::as_mut) was called when the value was
	/// either mutably or immutably borrowed.
	AlreadyLocked(Value),

	/// Mutable access on a [frozen value](crate::Value::freeze) was attempted.
	ValueFrozen(Value),

	/// Attempted access of the unknown attribute `attribute` on `object`.
	UnknownAttribute {
		object: Value,
		attribute: Value,
	},

	/// An `expected` type was required but a `given` was given.
	InvalidTypeGiven {
		expected: crate::value::Typename,
		given: crate::value::Typename,
	},

	/// The conversion function of `object` for type `into` was called, but the result wasn't
	/// something of type `into`.
	ConversionFailed {
		object: Value,
		into: crate::value::Typename,
	},

	/// For when i haven't made an actual error
	Message(String),

	/// Returns in quest are actually "Errors", although they don't have a stacktrace associated
	/// with them.
	Return {
		/// The value to return
		value: Value,
		/// The frame to return from, or `None` for the current frame.
		from_frame: Option<Value>,
	},

	/// A function expected no keyword arguments but they were given.
	KeywordsGivenWhenNotExpected,

	/// A function was given the wrong amount of arguments.
	PositionalArgumentMismatch {
		given: usize,
		expected: usize,
	},

	/// Attempted execution of a currently-running stackframe.
	StackframeIsCurrentlyRunning(crate::value::Gc<crate::vm::Frame>),

	/// Too many stackframes encountered
	StackOverflow,

	/// An assertion failed, with an optional message
	AssertionFailed(Option<crate::value::Gc<crate::value::ty::Text>>),

	// Division/Modulo/Exponentiation by an invalid value
	DivisionByZero(&'static str),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			write!(f, "error: {}\nstacktrace:\n{}", self.kind, self.stacktrace)
		} else {
			Display::fmt(&self.kind, f)
		}
	}
}

impl Display for ErrorKind {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::UnknownAttribute { object, attribute } => {
				write!(f, "unknown attribute {attribute:?} for {object:?}")
			}
			Self::AlreadyLocked(value) => write!(f, "value {value:?} is already locked"),
			Self::ValueFrozen(value) => write!(f, "value {value:?} is frozen"),
			Self::ConversionFailed { object, into } => {
				write!(f, "conversion {object:?} failed for {into:?}")
			}
			Self::InvalidTypeGiven { expected, given } => {
				write!(f, "invalid type {given:?}, expected {expected:?}")
			}
			Self::Message(msg) => f.write_str(msg),
			Self::Return { value, from_frame } => {
				write!(f, "returning value {value:?} from frame {from_frame:?}")
			}
			Self::KeywordsGivenWhenNotExpected => {
				write!(f, "keyword arguments given when none expected")
			}
			Self::PositionalArgumentMismatch { given, expected } => {
				write!(f, "positional argument count mismatch (given {given} expected {expected})")
			}
			Self::StackframeIsCurrentlyRunning(frame) => {
				write!(f, "frame {frame:?} is currently executing")
			}
			Self::StackOverflow => write!(f, "too many stackframes are running"),
			Self::AssertionFailed(None) => write!(f, "an assertion failed"),
			Self::AssertionFailed(Some(err)) => write!(f, "an assertion failed: {err:?}"),
			Self::DivisionByZero(kind) => write!(f, "{kind} by zero"),
		}
	}
}

impl From<String> for ErrorKind {
	fn from(msg: String) -> Self {
		Self::Message(msg)
	}
}

impl From<String> for Error {
	fn from(msg: String) -> Self {
		ErrorKind::from(msg).into()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Self {
		Self { kind, stacktrace: Stacktrace::current() }
	}
}

impl std::error::Error for Error {
	fn cause(&self) -> Option<&(dyn std::error::Error)> {
		None
	}
}
