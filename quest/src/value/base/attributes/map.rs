use super::Attribute;
use crate::{Intern, Result, ToValue, Value};
use hashbrown::hash_map::{Iter as HashBrownIter, RawEntryMut};
use hashbrown::HashMap;

/*
Under the (hopefully final) design, `Text::attr==` is the function that's used to compare a text
and another value for equality. This function is not able to be overwritten, just as `Text::hash` is
not.

Note that `Intern` shares the exact same implementation for `==` and `hash`.

There's three different options for looking up values:
	- If youre looking up an `Intern`
	- If youre looking up an `Text`
	- If youre looking up anything else

The `anythign else` is easy, because that'll never be an `Intern`.

The problem is `Intern`/`Text`. We cant convert all `Text`s into `Intern`ed values, as they might
have special attributes on them. As such, whenever you lookup an `Intern`, you first must check the
`Intern`s, and then check the uninterned ones, as it may or may not be there. Likewise, when you
lookup a `Text` dynamically, you must also check the `Intern`s.
*/

#[derive(Debug, Default)]
pub struct Map {
	interned: HashMap<Intern, Value>,
	any: HashMap<Value, Value>,
}

impl Map {
	pub fn with_capacity(capacity: usize) -> Box<Self> {
		Box::new(Self { interned: HashMap::with_capacity(capacity), any: HashMap::new() })
	}

	pub fn from_iter(iter: impl IntoIterator<Item = (Value, Value)>) -> Result<Box<Self>> {
		let mut map = Self::with_capacity(super::list::MAX_LISTMAP_LEN);

		for (attr, value) in iter {
			map.set_attr(attr, value)?;
		}

		Ok(map)
	}
}

pub struct MapIter<'a>(HashBrownIter<'a, Intern, Value>, HashBrownIter<'a, Value, Value>);

impl Iterator for MapIter<'_> {
	type Item = (Value, Value);

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((&key, &value)) = self.0.next() {
			Some((key.to_value(), value))
		} else if let Some((&key, &value)) = self.1.next() {
			Some((key, value))
		} else {
			None
		}
	}
}

impl Map {
	pub fn iter(&self) -> MapIter<'_> {
		MapIter(self.interned.iter(), self.any.iter())
	}

	pub fn len(&self) -> usize {
		self.interned.len() + self.any.len()
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		debug_assert!(!attr.is_special());

		if let Some(intern) = attr.as_intern()? {
			Ok(self.interned.get(&intern).copied())
		} else {
			self.get_unbound_any_attr(attr)
		}
	}

	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		let _ = self;
		let _ = attr;
		todo!("get unbound attr mut for maps");
		// debug_assert!(!attr.is_special());
	}

	fn get_unbound_any_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self.any.raw_entry().from_hash(hash, |&k| match attr.try_eq_value(k) {
			Ok(val) => val,
			Err(err) => {
				eq_err = Err(err);
				true
			}
		});
		eq_err?;

		Ok(res.map(|(_key, &val)| val))
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		debug_assert!(!attr.is_special());

		if let Some(intern) = attr.as_intern()? {
			self.interned.insert(intern, value);

			Ok(())
		} else {
			self.set_any_attr(attr, value)
		}
	}

	fn set_any_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self.any.raw_entry_mut().from_hash(hash, |&k| match attr.try_eq_value(k) {
			Ok(val) => val,
			Err(err) => {
				eq_err = Err(err);
				true
			}
		});
		eq_err?;

		match res {
			RawEntryMut::Occupied(mut occ) => {
				occ.insert(value);
			}
			RawEntryMut::Vacant(vac) => {
				vac.insert_with_hasher(hash, attr.to_value(), value, |k| {
					k.try_hash().expect("if the first hash doesn't fail, subsequent ones shouldn't")
				});
			}
		}

		Ok(())
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		debug_assert!(!attr.is_special());

		if let Some(_intern) = attr.as_intern()? {
			// You cannot remove interned values
			// TODO: is this actually the semantics we want?
			Ok(None)
		} else {
			self.del_any_attr(attr)
		}
	}

	fn del_any_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self.any.raw_entry_mut().from_hash(hash, |&k| match attr.try_eq_value(k) {
			Ok(val) => val,
			Err(err) => {
				eq_err = Err(err);
				true
			}
		});
		eq_err?;

		if let RawEntryMut::Occupied(occ) = res {
			Ok(Some(occ.remove()))
		} else {
			Ok(None)
		}
	}
}
