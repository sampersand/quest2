use super::Attribute;
use crate::value::AsAny;
use crate::value::Intern;
use crate::{AnyValue, Result};
use std::fmt::{self, Debug, Formatter};

pub const MAX_LISTMAP_LEN: usize = 32; //u8::BITS as usize;

#[derive(Clone, Copy)]
union Key {
	raw_data: u64,
	intern: Intern,
	value: AnyValue,
}

#[derive(Default)]
pub struct ListMap {
	data: [Option<(Key, AnyValue)>; MAX_LISTMAP_LEN],
	interned: u32,
}

impl Debug for ListMap {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut m = f.debug_map();

		for (idx, entry) in self.data.iter().enumerate() {
			if let Some((key, value)) = entry {
				if self.interned & (1 << idx) != 0 {
					m.entry(unsafe { &key.intern }, value);
				} else {
					m.entry(unsafe { &key.value }, value);
				}
			}
		}

		m.finish()
	}
}

macro_rules! is_eql_at {
	($list:ident, $idx:expr, $attr:expr, $value:expr) => {
		if $list.interned & (1 << $idx) != 0 {
			$attr.try_eq_intern(unsafe { $value.intern })?
		} else {
			$attr.try_eq_value(unsafe { $value.value })?
		}
	};
}

impl ListMap {
	pub fn is_full(&self) -> bool {
		self.data[MAX_LISTMAP_LEN - 1].is_some()
	}

	pub fn iter(&self) -> impl Iterator<Item = (AnyValue, AnyValue)> + '_ {
		struct Iter<'a>(&'a ListMap, usize);
		impl Iterator for Iter<'_> {
			type Item = (AnyValue, AnyValue);

			fn next(&mut self) -> Option<Self::Item> {
				if let Some((k, v)) = self.0.data.get(self.1).map(|x| *x).flatten() {
					let k = if self.0.interned & (1 << self.1) != 0 {
						unsafe { k.intern }.as_text().as_any()
					} else {
						unsafe { k.value }
					};
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
		debug_assert!(!attr.is_special());

		for (idx, &entry) in self.data.iter().enumerate() {
			if let Some((k, v)) = entry {
				if is_eql_at!(self, idx, attr, k) {
					return Ok(Some(v));
				}
			} else {
				break;
			}
		}

		Ok(None)
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		debug_assert!(!attr.is_special());

		for (idx, entry) in self.data.iter_mut().enumerate() {
			if let Some((k, v)) = entry {
				if is_eql_at!(self, idx, attr, k) {
					*v = value;
					// no need to update the interned value here, as the key doesnt change.
					return Ok(());
				}
			} else {
				let (raw_data, is_intern) = unsafe { attr.to_repr() };
				self.data[idx] = Some((Key { raw_data }, value));

				if is_intern {
					self.interned |= 1 << idx;
				}

				return Ok(());
			}
		}

		unreachable!("`set_attr` called when maxlen already reached?");
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		// this isn't terribly efficient, but then again most people aren't going to be
		// deleting things often, so it's alright.
		for (idx, entry) in self.data.iter_mut().enumerate() {
			if let Some((k, v)) = entry {
				if !is_eql_at!(self, idx, attr, k) {
					continue;
				}

				let value = *v;
				self.data[idx] = None;

				for j in idx + 1..MAX_LISTMAP_LEN {
					if self.data[j].is_none() {
						self.data.swap(idx, j - 1);
						break;
					}
				}

				self.interned &= !(1 << idx);

				return Ok(Some(value));
			} else {
				break;
			}
		}

		Ok(None)
	}
}
