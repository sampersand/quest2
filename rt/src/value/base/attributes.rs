use hashbrown::{HashMap, hash_map::RawEntryMut};
use crate::value::AnyValue;
use crate::Result;

mod parents;
pub use parents::Parents;

#[repr(C, align(8))]
#[derive(Debug, Default)]
pub(in super::super::super) struct Attributes {
	pub(in super::super) parents: Parents,
	attrs: Option<Box<HashMap<AnyValue, AnyValue>>>,
}

sa::assert_eq_size!(Attributes, [u64; 2]);
sa::assert_eq_align!(Attributes, [u64; 2]);

impl Attributes {
	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		if let Some(attrs) = &self.attrs {
			let hash = attr.try_hash()?;
			let mut eq_err: Result<()> = Ok(());

			let res = attrs.raw_entry().from_hash(hash, |&k| {
				match attr.try_eq(k) {
					Ok(val) => val,
					Err(err) => {
						eq_err = Err(err);
						true
					}
				}
			});
			eq_err?;

			if let Some((_key, &val)) = res {
				return Ok(Some(val))
			}
		}

		self.parents.get_attr(attr)
	}

	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		if self.attrs.is_none() {
			self.attrs = Some(Box::new(Default::default()));
		}

		let attrs = unsafe { self.attrs.as_mut().unwrap_unchecked() };

		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = attrs.raw_entry_mut().from_hash(hash, |&k| {
			match attr.try_eq(k) {
				Ok(val) => val,
				Err(err) => {
					eq_err = Err(err);
					true
				}
			}
		});
		eq_err?;

		match res {
			RawEntryMut::Occupied(mut occ) => {
				occ.insert(value);
			},
			RawEntryMut::Vacant(vac) => {
				// TODO: im not sure if the `|_| hash` is sound, because im not sure why it needs it
				// in the first place...
				vac.insert_with_hasher(hash, attr, value, |_| hash);
			},
		}

		Ok(())
	}

	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		let attrs = 
			if let Some(attrs) = &mut self.attrs {
				attrs
			} else {
				return Ok(None);
			};

		let hash = attr.try_hash()?;
		let mut eq_err: Result<()> = Ok(());

		let res = attrs.raw_entry_mut().from_hash(hash, |&k| {
			match attr.try_eq(k) {
				Ok(val) => val,
				Err(err) => {
					eq_err = Err(err);
					true
				}
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::{Value, Gc, ty::Integer};

	#[test]
	fn attributes_work() {
		let text = Gc::from_str("hola mundo");
		const ONE: AnyValue = Value::ONE.any();

		assert_matches!(text.as_ref().unwrap().get_attr(ONE), Ok(None));
		assert_matches!(text.as_mut().unwrap().del_attr(ONE), Ok(None));

		text.as_mut().unwrap().set_attr(ONE, Value::from(23).any()).unwrap();

		assert_eq!(text.as_ref().unwrap().get_attr(ONE).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 23);

		text.as_mut().unwrap().set_attr(ONE, Value::from(45).any()).unwrap();
		assert_eq!(text.as_ref().unwrap().get_attr(ONE).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 45);

		assert_eq!(text.as_mut().unwrap().del_attr(ONE).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 45);
		assert_matches!(text.as_ref().unwrap().get_attr(ONE), Ok(None));
	}

	#[test]
	fn parents_work() {
		const ATTR: AnyValue = Value::TRUE.any();

		let mut parent = Value::from("hello, world").any();
		parent.set_attr(ATTR, Value::from(123).any()).unwrap();
		assert_eq!(parent.get_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 123);

		let mut child = Value::ONE.any();
		assert!(!child.has_attr(ATTR).unwrap());

		child.parents().unwrap().as_mut().unwrap().push(parent);
		assert_eq!(child.get_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 123);

		child.set_attr(ATTR, Value::from(456).any()).unwrap();
		assert_eq!(child.get_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 456);

		assert_eq!(child.del_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 456);
		assert_eq!(child.get_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap().get(), 123);
		assert!(child.del_attr(ATTR).unwrap().is_none()); // cannot delete from parents.
	}
}

