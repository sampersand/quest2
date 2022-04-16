use super::{Attribute, InternKey};
use crate::value::AsAny;
use crate::{AnyValue, Result};
use std::fmt::{self, Debug, Formatter};

pub const MAX_LISTMAP_LEN: usize = 8;
#[derive(Clone, Copy)]
union Key {
	raw_data: u64,
	#[allow(dead_code)] // never explicitly read, it's read via `Intern::try_from_repr`.
	intern: InternKey,
	value: AnyValue,
}

#[derive(Default)]
pub(super) struct ListMap {
	data: [Option<(Key, AnyValue)>; MAX_LISTMAP_LEN],
}

macro_rules! if_intern {
	($key:expr, |$intern:ident| $ifi:expr, |$value:ident| $ifv:expr) => {{
		let key = $key;

		if let Some($intern) = InternKey::try_from_repr(unsafe { key.raw_data }).map(InternKey::as_intern) {
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
			if_intern!(key,
				|intern| m.entry(&intern, &value),
				|value| m.entry(&value, &value));
		}

		m.finish()
	}
}

impl Key {
	fn is_eql<A: Attribute>(self, attr: A) -> Result<bool> {
		if_intern!(self,
			|intern| attr.try_eq_intern(intern),
			|value| attr.try_eq_value(value))
	}
}

impl ListMap {
	pub fn is_full(&self) -> bool {
		self.data[MAX_LISTMAP_LEN - 1].is_some()
	}

	// Note that this drops the "intern"ness, but that's ok (i guess?)
	pub fn iter(&self) -> impl Iterator<Item = (AnyValue, AnyValue)> + '_ {
		struct Iter<'a>(&'a ListMap, usize);
		impl Iterator for Iter<'_> {
			type Item = (AnyValue, AnyValue);

			fn next(&mut self) -> Option<Self::Item> {
				if let Some((k, v)) = self.0.data.get(self.1).map(|x| *x).flatten() {
					let k = if_intern!(k, |intern| intern.as_text().as_any(), |value| value);
					self.1 += 1;
					Some((k, v))
				} else {
					None
				}
			}
		}

		Iter(self, 0)
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		for (key, value) in self.data.iter().map_while(|o| *o) {
			if key.is_eql(attr)? {
				return Ok(Some(value));
			}
		}

		Ok(None)
	}

	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut AnyValue> {
		for thing in self.data.iter_mut() {
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

	pub fn set_attr<A: Attribute>(&mut self, attr: A, new_value: AnyValue) -> Result<()> {
		for (idx, entry) in self.data.iter_mut().enumerate() {
			if let Some((key, value)) = entry {
				if !key.is_eql(attr)? {
					continue;
				}

				if let Some(intern) = InternKey::try_from_repr(unsafe { key.raw_data }) {
					if intern.is_frozen() {
						return Err(crate::Error::Message("attribute is frozen, cannot set it".to_string()))
					}
				}

				*value = new_value;
			} else {
				self.data[idx] = Some((Key { raw_data: attr.to_repr() }, new_value));
			}

			return Ok(());
		}

		unreachable!("`set_attr` called when maxlen already reached?");
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		// this isn't terribly efficient, but then again most people aren't going to be
		// deleting things often, so it's alright.
		for (idx, (key, value)) in self.data.iter_mut().map_while(|opt| *opt).enumerate() {
			if !key.is_eql(attr)? {
				continue;
			}

			if let Some(intern) = InternKey::try_from_repr(unsafe { key.raw_data }) {
				if intern.is_frozen() {
					return Err(crate::Error::Message("attribute is frozen, cannot set it".to_string()))
				}
			}

			self.data[idx] = None;

			// Find the last `None` element and swap it with the current one.
			for j in (idx + 1..=MAX_LISTMAP_LEN-1).rev() {
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
