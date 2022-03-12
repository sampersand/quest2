use crate::value::{Gc, base::Flags};
use crate::{AnyValue, Value, Result};
use crate::value::ty::Text;
use std::fmt::{self, Debug, Formatter};
use std::mem::ManuallyDrop;

mod list;
mod map;
use list::ListMap;
use map::Map;

#[repr(C, align(8))]
pub union Attributes {
	none: u64,
	list: ManuallyDrop<ListMap>,
	map: ManuallyDrop<Map>,
}

sa::assert_eq_size!(Attributes, u64);
sa::assert_eq_align!(Attributes, u64);

fn is_small(flags: &Flags) -> bool {
	!flags.contains(Flags::ATTR_MAP)
}

impl Attributes {
	// we're able to take `&mut self` as `0` is a valid variant—`none`.
	pub fn initialize_with_capacity(&mut self, capacity: usize) {
		if capacity == 0 {
			return;
		}

		if capacity <= list::MAX_LISTMAP_LEN {
			self.list = ManuallyDrop::new(ListMap::default());
		} else {
			self.map = ManuallyDrop::new(Map::with_capacity(capacity))
		}
	}


	const fn is_none(&self) -> bool {
		unsafe { self.none == 0 }
	}

	pub fn debug<'a>(&'a self, flags: &'a Flags) -> impl Debug + 'a {
		struct AttributesDebug<'a>(&'a Attributes, &'a Flags);

		impl Debug for AttributesDebug<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if self.0.is_none() {
					f.debug_map().finish()
				} else if is_small(self.1) {
					Debug::fmt(unsafe { &self.0.list }, f)
				} else {
					Debug::fmt(unsafe { &self.0.map }, f)
				}
			}
		}

		AttributesDebug(self, flags)
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A, flags: &Flags) -> Result<Option<AnyValue>> {
		if self.is_none() {
			Ok(None)
		} else if is_small(flags) {
			unsafe { &self.list }.get_unbound_attr(attr)
		} else {
			unsafe { &self.map }.get_unbound_attr(attr)
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue, flags: &Flags) -> Result<()> {
		if self.is_none() {
			self.list = ManuallyDrop::new(ListMap::default());
		}

		if is_small(flags) {
			unsafe {
				if !self.list.is_full() {
					return self.list.set_attr(attr, value);
				}

				self.map =
					ManuallyDrop::new(Map::from_iter(ManuallyDrop::take(&mut self.list).iter())?);
				flags.insert(Flags::ATTR_MAP);
			}
		}

		unsafe { &mut self.map }.set_attr(attr, value)
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A, flags: &Flags) -> Result<Option<AnyValue>> {
		if self.is_none() {
			Ok(None)
		} else if is_small(flags) {
			unsafe { &mut self.list }.del_attr(attr)
		} else {
			unsafe { &mut self.map }.del_attr(attr)
		}
	}

	pub unsafe fn drop(this: &mut Attributes, flags: &Flags) {
		if this.is_none() {
			// do nothing
		} else if is_small(flags) {
			ManuallyDrop::drop(&mut this.list)
		} else {
			ManuallyDrop::drop(&mut this.map)
		}
	}
}

pub trait Attribute : Copy + Debug {
	fn try_eq(self, rhs: AnyValue) -> Result<bool>;
	fn try_hash(self) -> Result<u64>;
	fn to_value(self) -> AnyValue;
}

impl Attribute for &'static str {
	fn try_eq(self, rhs: AnyValue) -> Result<bool> {
		if let Some(text) = rhs.downcast::<Gc<Text>>() {
			Ok(self == text.as_ref()?.as_str())
		} else {
			Ok(false)
		}
	}

	fn try_hash(self) -> Result<u64> {
		use std::hash::{Hash, Hasher};
		use std::collections::hash_map::DefaultHasher;

		let mut s = DefaultHasher::new();
		self.hash(&mut s);
		Ok(s.finish())
	}

	fn to_value(self) -> AnyValue {
		Value::from(Text::from_static_str(self)).any()
	}
}

impl Attribute for AnyValue {
	fn try_eq(self, rhs: AnyValue) -> Result<bool> {
		AnyValue::try_eq(self, rhs)
	}

	fn try_hash(self) -> Result<u64> {
		AnyValue::try_hash(self)
	}

	fn to_value(self) -> AnyValue {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::{
		ty::{Integer, Text},
		Value,
	};

	#[test]
	fn it_transitions_over_to_full_map() {
		let text = Text::from_str("yo waddup");
		let mut textmut = text.as_mut().unwrap();

		for i in 0..=list::MAX_LISTMAP_LEN * 2 {
			let value = Value::from(i as i64).any();
			textmut.set_attr(value, value).unwrap();

			// assert!(textmut.r().get_unbound_attr(value).unwrap().unwrap().try_eq(value).unwrap());
		}

		let value = Value::from(3).any();
		assert!(textmut
			.r()
			.get_unbound_attr(value)
			.unwrap()
			.unwrap()
			.try_eq(value)
			.unwrap());

		// now it should be a full `map`, let's go over all of them again.
		for i in 0..=list::MAX_LISTMAP_LEN * 2 {
			let value = Value::from(i as i64).any();
			assert!(textmut
				.r()
				.get_unbound_attr(value)
				.unwrap()
				.unwrap()
				.try_eq(value)
				.unwrap());
		}
	}

	#[test]
	fn attributes_work() {
		let text = Text::from_str("hola mundo");
		const ONE: AnyValue = Value::ONE.any();

		assert_matches!(text.as_ref().unwrap().get_unbound_attr(ONE), Ok(None));
		assert_matches!(text.as_mut().unwrap().del_attr(ONE), Ok(None));

		text
			.as_mut()
			.unwrap()
			.set_attr(ONE, Value::from(23).any())
			.unwrap();

		assert_eq!(
			text
				.as_ref()
				.unwrap()
				.get_unbound_attr(ONE)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			23
		);

		text
			.as_mut()
			.unwrap()
			.set_attr(ONE, Value::from(45).any())
			.unwrap();
		assert_eq!(
			text
				.as_ref()
				.unwrap()
				.get_unbound_attr(ONE)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			45
		);

		assert_eq!(
			text
				.as_mut()
				.unwrap()
				.del_attr(ONE)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			45
		);
		assert_matches!(text.as_ref().unwrap().get_unbound_attr(ONE), Ok(None));
	}

	#[test]
	fn parents_work() {
		const ATTR: AnyValue = Value::TRUE.any();

		let mut parent = Value::from("hello, world").any();
		parent.set_attr(ATTR, Value::from(123).any()).unwrap();
		assert_eq!(
			parent
				.get_unbound_attr(ATTR)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			123
		);

		let mut child = Value::ONE.any();
		assert!(!child.has_attr(ATTR).unwrap());

		child.parents().unwrap().as_mut().unwrap().push(parent);
		assert_eq!(
			child
				.get_unbound_attr(ATTR)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			123
		);

		child.set_attr(ATTR, Value::from(456).any()).unwrap();
		assert_eq!(
			child
				.get_unbound_attr(ATTR)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			456
		);

		assert_eq!(
			child
				.del_attr(ATTR)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			456
		);
		assert_eq!(
			child
				.get_unbound_attr(ATTR)
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			123
		);
		assert!(child.del_attr(ATTR).unwrap().is_none()); // cannot delete from parents.
	}
}
