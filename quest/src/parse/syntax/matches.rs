use hashbrown::HashMap;
use crate::parse::{Token, Parser, Result};

#[derive(Debug)]
pub struct Matcher<'tkn, 'vec, 'caps> {
	all_tokens: &'vec mut Vec<Token<'tkn>>,
	start_index: usize, // into `all_tokens`
	captures: Captures<'tkn, 'caps>
}

#[derive(Debug)]
enum Captures<'tkn, 'caps> {
	Owned(HashMap<&'tkn str, Vec<Matches<'tkn>>>),
	Borrowed(&'caps mut HashMap<&'tkn str, Vec<Matches<'tkn>>>)
}

impl<'tkn, 'vec, 'caps> Matcher<'tkn, 'vec, 'caps> {
	pub fn new(all_tokens: &'vec mut Vec<Token<'tkn>>) -> Self {
		Self {
			all_tokens,
			start_index: 0,
			captures: Captures::Owned(HashMap::default())
		}
	}

	pub fn submatcher(&mut self) -> Matcher<'tkn, '_, '_> {
		Matcher {
			start_index: self.all_tokens.len(),
			all_tokens: self.all_tokens,
			captures: match &mut self.captures {
				Captures::Owned(owned) => Captures::Borrowed(owned),
				Captures::Borrowed(borrowed) => Captures::Borrowed(borrowed),
			}
		}
	}

	pub fn _subnesting(&mut self) -> Matcher<'tkn, '_, '_> {
		Matcher {
			start_index: self.all_tokens.len(),
			all_tokens: self.all_tokens,
			captures: Captures::Owned(HashMap::default())
		}
	}

	pub fn push(&mut self, token: Token<'tkn>) {
		self.all_tokens.push(token);
	}

	pub fn declare_capture(&mut self, name: &'tkn str, matches: Vec<Matches<'tkn>>) -> Result<'tkn, ()> {
		// we can ignore unnamed captures (i think?)
		/*if name == "_" {
			return Ok(());
		}*/

		let start = matches[0].all_tokens[0].span.start;
		let old = match &mut self.captures {
			Captures::Owned(owned) => owned.insert(name, matches),
			Captures::Borrowed(borrowed) => borrowed.insert(name, matches),
		};

		if old.is_none() {
			Ok(())
		} else {
			// todo: should the error originate from the syntax token?
			Err(start.error(format!("duplicate syntax variable '${}' encountered", name).into()))
		}
	}

	pub fn unmatch(self, parser: &mut Parser<'tkn>) {
		parser.untake_tokens(self.all_tokens[self.start_index..].iter().copied());

		// todo: optimize me
		let amnt_to_take = self.all_tokens.len() - self.start_index;
		for _ in 0..amnt_to_take {
			self.all_tokens.pop();
		}
	}

	pub fn finish(self) -> Matches<'tkn> {
		let all_tokens = self.all_tokens[self.start_index..].iter().copied().collect::<Vec<_>>();

		Matches {
			all_tokens,
			captures: match self.captures {
				Captures::Owned(owned) => owned,
				Captures::Borrowed(_borrowed) => Default::default(),
			}
		}
	}
}


#[derive(Debug, Default)]
pub struct Matches<'tkn> {
	all_tokens: Vec<Token<'tkn>>,
	captures: HashMap<&'tkn str, Vec<Matches<'tkn>>>,
}


impl<'tkn> Matches<'tkn> {
	pub fn all_tokens(&self) -> &[Token<'tkn>] {
		&self.all_tokens
	}

	pub fn capture(&self, name: &str) -> Option<&[Matches<'tkn>]> {
		self.captures.get(name).map(Vec::as_slice)
	}
}
