use super::Attribute;
use crate::{Intern, Result, ToValue, Value};
use std::fmt::{self, Debug, Formatter};

pub const MAX_LISTMAP_LEN: usize = 8;
#[derive(Clone, Copy)]
union Key {
	raw_data: u64,
	#[allow(dead_code)] // never explicitly read, it's read via `Intern::try_from_repr`.
	intern: Intern,
	value: Value,
}

pub(super) struct ListMap {
	data: [Option<(Key, Value)>; MAX_LISTMAP_LEN],
}

macro_rules! if_intern {
	($key:expr, |$intern:ident| $ifi:expr, |$value:ident| $ifv:expr) => {{
		let key = $key;

		if let Some($intern) = unsafe { Intern::try_from_repr(key.raw_data) } {
			$ifi
		} else {
			let $value = unsafe { key.value };
			$ifv
		}
	}};
}

impl Debug for ListMap {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut m = f.debug_map();

		for (key, value) in self.data.iter().map_while(|o| *o) {
			if_intern!(key, |intern| m.entry(&intern, &value), |value| m.entry(&value, &value));
		}

		m.finish()
	}
}

impl Key {
	fn is_eql<A: Attribute>(self, attr: A) -> Result<bool> {
		if_intern!(self, |intern| attr.try_eq_intern(intern), |value| attr.try_eq_value(value))
	}
}

pub struct ListMapIter<'a>(&'a ListMap, usize);

impl Iterator for ListMapIter<'_> {
	type Item = (Value, Value);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((k, v)) = self.0.data.get(self.1).copied().flatten() {
			let k = if_intern!(k, |intern| intern.as_text().to_value(), |value| value);
			self.1 += 1;
			Some((k, v))
		} else {
			None
		}
	}
}

impl ListMap {
	pub fn new() -> Box<Self> {
		Box::new(Self { data: [None; MAX_LISTMAP_LEN] })
	}

	pub fn iter(&self) -> ListMapIter<'_> {
		ListMapIter(self, 0)
	}

	pub fn is_full(&self) -> bool {
		self.data[MAX_LISTMAP_LEN - 1].is_some()
	}

	pub fn len(&self) -> usize {
		self.data.iter().take_while(|v| v.is_some()).count()
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		for (key, value) in self.data.iter().map_while(|o| *o) {
			if key.is_eql(attr)? {
				return Ok(Some(value));
			}
		}

		Ok(None)
	}

	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		for thing in &mut self.data {
			if let Some((key, value)) = thing {
				if key.is_eql(attr)? {
					return Ok(value);
				}
			} else {
				break;
			}
		}

		panic!("get_unbound_attr_mut called with an unknown attribute");
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, new_value: Value) -> Result<()> {
		for (idx, entry) in self.data.iter_mut().enumerate() {
			if let Some((key, value)) = entry {
				if !key.is_eql(attr)? {
					continue;
				}

				*value = new_value;
			} else {
				self.data[idx] = Some((Key { raw_data: attr.bits() }, new_value));
			}

			return Ok(());
		}

		unreachable!("`set_attr` called when maxlen already reached?");
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		// this isn't terribly efficient, but then again most people aren't going to be
		// deleting things often, so it's alright.
		for (idx, (key, value)) in self.data.iter_mut().map_while(|opt| *opt).enumerate() {
			if !key.is_eql(attr)? {
				continue;
			}

			self.data[idx] = None;

			// Find the last `None` element and swap it with the current one.
			for j in (idx + 1..MAX_LISTMAP_LEN).rev() {
				if self.data[j].is_none() {
					self.data.swap(idx, j - 1);
					break;
				}
			}

			return Ok(Some(value));
		}

		Ok(None)
	}
}
