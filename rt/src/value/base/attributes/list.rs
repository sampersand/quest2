use crate::AnyValue;
use crate::Result;

pub const MAX_LISTMAP_LEN: usize = 8;

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct ListMap(Box<[Option<(AnyValue, AnyValue)>; MAX_LISTMAP_LEN]>);

impl ListMap {
	pub fn is_full(&self) -> bool {
		self.0[MAX_LISTMAP_LEN - 1].is_some()
	}

	pub fn iter(&self) -> impl Iterator<Item = (AnyValue, AnyValue)> + '_ {
		struct Iter<'a>(&'a [Option<(AnyValue, AnyValue)>]);
		impl Iterator for Iter<'_> {
			type Item = (AnyValue, AnyValue);

			fn next(&mut self) -> Option<Self::Item> {
				if let Some(&ele) = self.0.get(0) {
					self.0 = &self.0[1..];
					ele
				} else {
					None
				}
			}
		}

		Iter(&*self.0)
	}

	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		for i in 0..MAX_LISTMAP_LEN {
			if let Some((k, v)) = self.0[i] {
				if attr.try_eq(k)? {
					return Ok(Some(v));
				}
			} else {
				break;
			}
		}

		Ok(None)
	}

	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		for i in 0..MAX_LISTMAP_LEN {
			if let Some((k, v)) = &mut self.0[i] {
				if attr.try_eq(*k)? {
					*v = value;
					return Ok(());
				}
			} else {
				self.0[i] = Some((attr, value));
				return Ok(());
			}
		}

		unreachable!("`set_attr` called when maxlen already reached?");
	}

	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		// this isn't terribly efficient, but then again most people aren't going to be
		// deleting things often, so it's alright.
		for i in 0..MAX_LISTMAP_LEN {
			if let Some((k, v)) = &mut self.0[i] {
				if attr.try_eq(*k)? {
					let value = *v;
					self.0[i] = None;

					for j in i + 1..MAX_LISTMAP_LEN {
						if self.0[j].is_none() {
							self.0.swap(i, j - 1);
							break;
						}
					}

					return Ok(Some(value));
				}
			} else {
				break;
			}
		}

		Ok(None)
	}
}
