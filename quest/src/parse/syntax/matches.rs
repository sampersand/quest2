use crate::parse::{Parser, Result, Token};
use hashbrown::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Debug)]
pub struct Matcher<'tkn, 'vec, 'caps> {
	all_tokens: &'vec mut Vec<Token<'tkn>>,
	start_index: usize, // into `all_tokens`
	captures: Shared<'caps, HashMap<&'tkn str, Vec<Matches<'tkn>>>>,
	sequences: Shared<'caps, HashMap<&'tkn str, Vec<Rc<[Matches<'tkn>]>>>>,
}

#[derive(Debug)]
enum Shared<'caps, T> {
	Owned(T),
	Borrowed(&'caps mut T),
}

impl<T> Deref for Shared<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(owned) => owned,
			Self::Borrowed(borrowed) => borrowed,
		}
	}
}

impl<T> DerefMut for Shared<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Owned(ref mut owned) => owned,
			Self::Borrowed(borrowed) => borrowed,
		}
	}
}

impl<'tkn, 'vec, 'caps> Matcher<'tkn, 'vec, 'caps> {
	pub fn new(all_tokens: &'vec mut Vec<Token<'tkn>>) -> Self {
		Self {
			all_tokens,
			start_index: 0,
			captures: Shared::Owned(HashMap::new()),
			sequences: Shared::Owned(HashMap::new()),
		}
	}

	pub fn submatcher(&mut self) -> Matcher<'tkn, '_, '_> {
		Matcher {
			start_index: self.all_tokens.len(),
			all_tokens: self.all_tokens,
			captures: Shared::Borrowed(&mut self.captures),
			sequences: Shared::Borrowed(&mut self.sequences),
		}
	}

	pub fn subpattern(&mut self) -> Matcher<'tkn, '_, '_> {
		Matcher {
			start_index: self.all_tokens.len(),
			all_tokens: self.all_tokens,
			captures: Shared::Owned(HashMap::default()),
			sequences: Shared::Owned(HashMap::default()),
		}
	}

	pub fn push(&mut self, token: Token<'tkn>) {
		self.all_tokens.push(token);
	}

	fn named_capture_defined(&self, name: &str) -> bool {
		for (key, caps) in &*self.captures {
			if *key == name || caps.iter().any(|cap| cap.named_defined(name)) {
				return true;
			}
		}
		false
	}

	fn named_defined(&self, name: &str) -> bool {
		if self.named_capture_defined(name) {
			return true;
		}

		for (key, caps) in &*self.sequences {
			if *key == name || caps.iter().any(|cap| cap.iter().any(|c| c.named_defined(name))) {
				return true;
			}
		}
		false
	}

	pub fn declare_capture(
		&mut self,
		name: &'tkn str,
		matches: Vec<Matches<'tkn>>,
	) -> Result<'tkn, ()> {
		if name == "_" {
			return Ok(());
		}

		if self.named_defined(name) {
			return Err(
				matches[0].all_tokens[0]
					.span
					.start
					.error(format!("duplicate syntax variable '${name}' encountered").into()),
			);
		}

		self.captures.insert(name, matches);
		Ok(())
	}

	pub fn declare_submatches(&mut self, submatches: Vec<Matches<'tkn>>) -> Result<'tkn, ()> {
		let submatches = Rc::<[Matches<'tkn>]>::from(submatches);

		for submatch in submatches.iter() {
			// only look thru keys, subsubmatches dont count for new vars
			for &name in submatch.captures.keys() {
				if name == "_" {
					continue;
				}

				if !self.named_capture_defined(name) && !self.sequences.contains_key(name) {
					self.sequences.entry(name).or_default().push(submatches.clone());
				}
			}
		}

		Ok(())
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
		let all_tokens = self.all_tokens[self.start_index..].to_vec();

		Matches {
			all_tokens,
			captures: match self.captures {
				Shared::Owned(owned) => owned,
				Shared::Borrowed(_borrowed) => Default::default(),
			},
			sequences: match self.sequences {
				Shared::Owned(owned) => owned,
				Shared::Borrowed(_borrowed) => Default::default(),
			},
		}
	}
}

#[derive(Debug, Default)]
pub struct Matches<'tkn> {
	all_tokens: Vec<Token<'tkn>>,
	captures: HashMap<&'tkn str, Vec<Matches<'tkn>>>,
	sequences: HashMap<&'tkn str, Vec<Rc<[Matches<'tkn>]>>>,
}

impl<'tkn> Matches<'tkn> {
	fn named_defined(&self, _name: &str) -> bool {
		false // todo
	}

	pub fn all_tokens(&self) -> &[Token<'tkn>] {
		&self.all_tokens
	}

	pub fn expand(&self, parser: &mut Parser<'tkn>) {
		parser.untake_tokens(self.all_tokens().iter().copied());
	}

	pub fn capture(&self, name: &str) -> Option<&[Matches<'tkn>]> {
		self.captures.get(name).map(Vec::as_slice)
	}

	pub fn get_submatches_with(&self, name: &str) -> Option<&[Rc<[Matches<'tkn>]>]> {
		self.sequences.get(name).map(|x| x.as_slice())
	}
}
