use crate::value::ToAny;
use crate::{AnyValue, Result};

/// Arguments passed to native Quest functions.
#[derive(Default, Debug, Clone, Copy)]
pub struct Args<'a> {
	positional: &'a [AnyValue],
	keyword: &'a [(&'a str, AnyValue)],
	this: Option<AnyValue>,
}

impl<'a> Args<'a> {
	/// Creates a new [`Args`] with the given positional and keyword arguments
	#[must_use]
	pub const fn new(positional: &'a [AnyValue], keyword: &'a [(&'a str, AnyValue)]) -> Self {
		Self {
			positional,
			keyword,
			this: None,
		}
	}

	/// Creates a new [`Args`] with the same positional and keyword arguments as `self`, except with
	/// the `this` field set to `this`.
	///
	/// While it's not invalid to call `with_this` when [`self`] already has a `this`, it is
	/// indicative of a logic bug, and will panic on debug builds.
	#[must_use]
	pub const fn with_this(self, this: AnyValue) -> Self {
		debug_assert!(self.this.is_none(), "todo: is this even possible? and if so, how should it work");

		Self {
			this: Some(this),
			..self
		}
	}

	/// Returns this `this` associated with `self`, if any.
	#[must_use]
	pub const fn this(self) -> Option<AnyValue> {
		self.this
	}

	/// Returns the list of positional arguments.
	#[must_use]
	pub const fn positional(self) -> &'a [AnyValue] {
		self.positional
	}

	/// Returns the list of keyword arguments.
	#[must_use]
	pub const fn keyword(self) -> &'a [(&'a str, AnyValue)] {
		self.keyword
	}

	/// Returns the amount of arguments total that were passed
	#[must_use]
	pub const fn len(self) -> usize {
		self.positional.len() + self.keyword.len() + if self.this.is_some() { 1 } else { 0 }
	}

	/// Checks to see whether `self` is empty.
	#[must_use]
	pub const fn is_empty(self) -> bool {
		self.len() == 0
	}

	/// <TODO: eventually remove this.>
	#[deprecated]
	pub fn idx_err_unless(self, func: impl FnOnce(Self) -> bool) -> Result<Self> {
		if func(self) {
			Ok(self)
		} else {
			Err(crate::error::ErrorKind::Message("argument count error error happened".into()).into())
		}
	}

	/// Fetches the argument specified by `index`, returning `None` if it isn't defined.
	pub fn get<T: ArgIndexer>(self, index: T) -> Option<AnyValue> {
		index.get(self)
	}

	/// Asserts there are no positional arguments.
	pub fn assert_no_positional(self) -> Result<()> {
		if self.positional.is_empty() {
			Ok(())
		} else {
			Err(crate::error::ErrorKind::Message("positional arguments given when none expected".to_string()).into())
		}
	}

	/// Asserts the positional arguments are exactly `len` long.
	pub fn assert_positional_len(self, len: usize) -> Result<()> {
		if self.positional.len() == len {
			Ok(())
		} else {
			Err(crate::error::ErrorKind::PositionalArgumentMismatch {
				given: len,
				expected: self.positional.len(),
			}.into())
		}
	}

	/// Asserts there are no keyowrd arguments.
	pub fn assert_no_keyword(self) -> Result<()> {
		if self.keyword.is_empty() {
			Ok(())
		} else {
			Err(crate::error::ErrorKind::KeywordsGivenWhenNotExpected.into())
		}
	}

	/// Asserts there are arguments whatsoever.
	pub fn assert_no_arguments(self) -> Result<()> {
		self.assert_no_positional()?;
		self.assert_no_keyword()?;

		Ok(())
	}

	/// Returns the first argument (or `this` if it's supplied) and the rest of them.
	pub fn split_first(mut self) -> Result<(AnyValue, Self)> {
		if let Some(this) = self.this.take() {
			return Ok((this, self));
		}

		self.idx_err_unless(|a| !a.is_empty())?;

		Ok((self[0], Self::new(&self.positional[1..], self.keyword)))
	}

	/// Converts `self` into a value.
	#[must_use]
	pub fn into_value(self) -> AnyValue {
		self
			.assert_no_keyword()
			.expect("todo: keyword for argument into value");

		let mut builder = crate::value::ty::List::builder();

		let mut len = self.positional.len();
		if self.this.is_some() {
			len += 1;
		}

		unsafe {
			builder.allocate_buffer(len);
			if let Some(this) = self.this {
				builder.list_mut().push(this);
			}

			builder.list_mut().push_slice_unchecked(self.positional);
			builder.finish().to_any()
		}
	}
}

impl<A: ArgIndexer> std::ops::Index<A> for Args<'_> {
	type Output = AnyValue;

	fn index(&self, idx: A) -> &Self::Output {
		idx.index(*self)
	}
}

/// A helper trait to allow indexing into [`Args`] with different types.
pub trait ArgIndexer {
	/// Try to fetch `self` from `args`.
	fn get(self, args: Args<'_>) -> Option<AnyValue>;
	
	/// Fetches `self` from `args`, `panic!`ing if it doesn't exist in `args`.
	fn index(self, args: Args<'_>) -> &AnyValue;
}

impl ArgIndexer for usize {
	fn get(self, args: Args<'_>) -> Option<AnyValue> {
		args.positional.get(self).copied()
	}

	fn index(self, args: Args<'_>) -> &AnyValue {
		&args.positional[self]
	}
}

impl ArgIndexer for &'static str {
	fn get(self, args: Args<'_>) -> Option<AnyValue> {
		for &(kw, val) in args.keyword {
			if kw == self {
				return Some(val);
			}
		}

		None
	}

	fn index(self, args: Args<'_>) -> &AnyValue {
		for (kw, val) in args.keyword {
			if *kw == self {
				return val;
			}
		}

		panic!("variable {self:?} doesnt exist in {args:?}");
	}
}
