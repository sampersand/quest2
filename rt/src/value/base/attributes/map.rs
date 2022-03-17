use crate::{AnyValue, Result};
use crate::value::Intern;
use super::Attribute;
use hashbrown::{hash_map::RawEntryMut, HashMap};

#[derive(Debug, Default)]
pub struct Map {
	interned: HashMap<Intern, AnyValue>,
	any: HashMap<AnyValue, AnyValue>
}

impl Map {
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			interned: HashMap::with_capacity(capacity),
			any: HashMap::new()
		}
	}

	pub fn from_iter(iter: impl IntoIterator<Item = (AnyValue, AnyValue)>) -> Result<Self> {
		let mut map = Self::with_capacity(super::list::MAX_LISTMAP_LEN);

		for (attr, value) in iter {
			map.set_attr(attr, value)?;
		}

		Ok(map)
	}
}

impl Map {
	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self
			.any
			.raw_entry()
			.from_hash(hash, |&k| match attr.try_eq_value(k) {
				Ok(val) => val,
				Err(err) => {
					eq_err = Err(err);
					true
				},
			});
		eq_err?;

		Ok(res.map(|(_key, &val)| val))
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		debug_assert!(!attr.is_special());

		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self
			.any
			.raw_entry_mut()
			.from_hash(hash, |&k| match attr.try_eq_value(k) {
				Ok(val) => val,
				Err(err) => {
					eq_err = Err(err);
					true
				},
			});
		eq_err?;

		match res {
			RawEntryMut::Occupied(mut occ) => {
				occ.insert(value);
			},
			RawEntryMut::Vacant(vac) => {
				vac.insert_with_hasher(hash, attr.to_value(), value, |k| {
					k.try_hash()
						.expect("if the first hash doesn't fail, subsequent ones shouldn't")
				});
			},
		}

		Ok(())
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = self
			.any
			.raw_entry_mut()
			.from_hash(hash, |&k| match attr.try_eq_value(k) {
				Ok(val) => val,
				Err(err) => {
					eq_err = Err(err);
					true
				},
			});
		eq_err?;

		if let RawEntryMut::Occupied(occ) = res {
			Ok(Some(occ.remove()))
		} else {
			Ok(None)
		}
	}
}
