use crate::{AnyValue, Error, Result};

#[derive(Default, Debug, Clone, Copy)]
pub struct Args<'a> {
	positional: &'a [AnyValue],
	keyword: &'a [(&'a str, AnyValue)],
}

impl<'a> Args<'a> {
	pub const fn new(positional: &'a [AnyValue], keyword: &'a [(&'a str, AnyValue)]) -> Self {
		Self {
			positional,
			keyword,
		}
	}

	pub const fn positional(self) -> &'a [AnyValue] {
		self.positional
	}

	pub const fn keyword(self) -> &'a [(&'a str, AnyValue)] {
		self.keyword
	}

	pub const fn len(self) -> usize {
		self.positional.len() + self.keyword.len()
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

	pub fn assert_no_positional(self) -> Result<Self> {
		if self.positional.is_empty() {
			Ok(self)
		} else {
			Err(Error::Message("positional arguments given when none expected".to_string()))
		}
	}

	pub fn assert_no_keyword(self) -> Result<Self> {
		if self.keyword.is_empty() {
			Ok(self)
		} else {
			Err(Error::Message("keyword arguments given when none expected".to_string()))
		}
	}

	pub fn assert_no_arguments(self) -> Result<Self> {
		self.assert_no_positional()?.assert_no_keyword()
	}
}

pub trait ArgIndexer {
	fn get(self, args: Args<'_>) -> Result<AnyValue>;
}

impl ArgIndexer for usize {
	fn get(self, args: Args<'_>) -> Result<AnyValue> {
		args
			.positional
			.get(self)
			.cloned()
			.ok_or(Error::MissingPositionalArgument(self))
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
}
