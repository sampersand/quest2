use crate::{AnyValue, Error, Result};

#[derive(Default, Debug, Clone, Copy)]
pub struct Args<'a> {
	positional: &'a [AnyValue],
	keyword: &'a [(&'a str, AnyValue)],
	this: Option<AnyValue>,
}

impl<'a> Args<'a> {
	pub const fn new(positional: &'a [AnyValue], keyword: &'a [(&'a str, AnyValue)]) -> Self {
		Self {
			positional,
			keyword,
			this: None,
		}
	}

	pub fn with_self(self, this: AnyValue) -> Self {
		assert!(
			!self.this.is_some(),
			"todo: is this even possible? and if so, how should it work"
		);

		Self {
			this: Some(this),
			..self
		}
	}

	pub fn get_self(self) -> Option<AnyValue> {
		self.this
	}

	pub const fn positional(self) -> &'a [AnyValue] {
		self.positional
	}

	pub const fn keyword(self) -> &'a [(&'a str, AnyValue)] {
		self.keyword
	}

	pub const fn len(self) -> usize {
		self.positional.len() + self.keyword.len() + if self.this.is_some() { 1 } else { 0 }
	}

	pub const fn is_empty(self) -> bool {
		self.len() == 0
	}

	pub fn idx_err_unless(self, func: impl FnOnce(Self) -> bool) -> Result<Self> {
		if func(self) {
			Ok(self)
		} else {
			Err(Error::Message("index error happened".into()))
		}
	}

	pub fn get<T: ArgIndexer>(self, index: T) -> Result<AnyValue> {
		index.get(self)
	}

	pub fn assert_no_positional(self) -> Result<()> {
		if self.positional.is_empty() {
			Ok(())
		} else {
			Err(Error::Message("positional arguments given when none expected".to_string()))
		}
	}

	pub fn assert_positional_len(self, len: usize) -> Result<()> {
		if self.positional.len() == len {
			Ok(())
		} else {
			Err(Error::Message(format!(
				"positional argument count mismatch (given {} expected {})",
				len,
				self.positional.len()
			)))
		}
	}
	pub fn assert_no_keyword(self) -> Result<()> {
		if self.keyword.is_empty() {
			Ok(())
		} else {
			Err(Error::Message("keyword arguments given when none expected".to_string()))
		}
	}

	pub fn assert_no_arguments(self) -> Result<()> {
		self.assert_no_positional()?;
		self.assert_no_keyword()?;

		Ok(())
	}

	pub fn split_first(mut self) -> Result<(AnyValue, Self)> {
		if let Some(this) = self.this.take() {
			return Ok((this, self));
		}

		self.idx_err_unless(|a| a.len() >= 1)?;

		Ok((self[0], Self::new(&self.positional[1..], self.keyword)))
	}
}

impl<A: ArgIndexer> std::ops::Index<A> for Args<'_> {
	type Output = AnyValue;

	fn index(&self, idx: A) -> &Self::Output {
		idx.index(*self)
	}
}

pub trait ArgIndexer {
	fn get(self, args: Args<'_>) -> Result<AnyValue>;
	fn index<'a>(self, args: Args<'a>) -> &'a AnyValue;
}

impl ArgIndexer for usize {
	fn get(self, args: Args<'_>) -> Result<AnyValue> {
		args
			.positional
			.get(self)
			.cloned()
			.ok_or(Error::MissingPositionalArgument(self))
	}

	fn index<'a>(self, args: Args<'a>) -> &'a AnyValue {
		&args.positional[self]
	}
}

impl ArgIndexer for &'static str {
	fn get(self, args: Args<'_>) -> Result<AnyValue> {
		for &(kw, val) in args.keyword {
			if kw == self {
				return Ok(val);
			}
		}

		Err(Error::MissingKeywordArgument(self))
	}

	fn index<'a>(self, args: Args<'a>) -> &'a AnyValue {
		for (kw, val) in args.keyword {
			if *kw == self {
				return val;
			}
		}

		panic!("variable {:?} doesnt exist in {:?}", self, args);
	}
}
