use crate::value::ty::Text;
use crate::value::{base::Flags, AsAny, Gc, Intern};
use crate::{AnyValue, Result};
use std::fmt::{self, Debug, Formatter};
use std::mem::ManuallyDrop;

mod list;
mod map;
use list::ListMap;
use map::Map;

pub union Attributes {
	list: ManuallyDrop<ListMap>,
	map: ManuallyDrop<Map>,
}

fn is_small(flags: &Flags) -> bool {
	!flags.contains(Flags::ATTR_MAP)
}

impl Attributes {
	pub fn new(flags: &Flags) -> Self {
		debug_assert!(is_small(flags)); // shouldn't be set to begin with

		Self {
			list: ManuallyDrop::new(ListMap::default()),
		}
	}

	// we're able to take `&mut self` as `0` is a valid variant—`none`.
	pub fn with_capacity(capacity: usize, flags: &Flags) -> Self {
		debug_assert_ne!(capacity, 0); // with 0 capacity, `Attributes` shouldn't be allocated

		if capacity <= list::MAX_LISTMAP_LEN {
			Self {
				list: ManuallyDrop::new(ListMap::default()),
			}
		} else {
			flags.insert(Flags::ATTR_MAP);
			Self {
				map: ManuallyDrop::new(Map::with_capacity(capacity)),
			}
		}
	}

	pub unsafe fn debug<'a>(&'a self, flags: &'a Flags) -> impl Debug + 'a {
		struct AttributesDebug<'a>(&'a Attributes, &'a Flags);

		impl Debug for AttributesDebug<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if is_small(self.1) {
					Debug::fmt(unsafe { &*self.0.list }, f)
				} else {
					Debug::fmt(unsafe { &*self.0.map }, f)
				}
			}
		}

		AttributesDebug(self, flags)
	}

	pub fn get_unbound_attr<A: Attribute>(
		&self,
		attr: A,
		flags: &Flags,
	) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		if is_small(flags) {
			unsafe { &self.list }.get_unbound_attr(attr)
		} else {
			unsafe { &self.map }.get_unbound_attr(attr)
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue, flags: &Flags) -> Result<()> {
		debug_assert!(!attr.is_special());

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
		debug_assert!(!attr.is_special());

		if is_small(flags) {
			unsafe { &mut self.list }.del_attr(attr)
		} else {
			unsafe { &mut self.map }.del_attr(attr)
		}
	}

	pub unsafe fn drop(this: &mut Attributes, flags: &Flags) {
		if is_small(flags) {
			ManuallyDrop::drop(&mut this.list)
		} else {
			ManuallyDrop::drop(&mut this.map)
		}
	}
}

pub trait Attribute: Copy + Debug {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool>;
	fn try_eq_intern(self, rhs: Intern) -> Result<bool>;

	fn try_hash(self) -> Result<u64>;
	fn to_value(self) -> AnyValue;
	unsafe fn to_repr(self) -> (u64, bool);

	fn is_parents(self) -> bool;
	fn is_special(self) -> bool {
		self.is_parents()
	}
}

impl Attribute for crate::value::Intern {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool> {
		if let Some(text) = rhs.downcast::<Gc<Text>>() {
			Ok(&*self == text.as_ref()?.as_str())
		} else {
			Ok(false)
		}
	}

	fn try_eq_intern(self, rhs: Intern) -> Result<bool> {
		Ok(self == rhs)
	}

	fn try_hash(self) -> Result<u64> {
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};

		let mut s = DefaultHasher::new();
		self.hash(&mut s);
		Ok(s.finish())
	}

	fn to_value(self) -> AnyValue {
		self.as_text().as_any()
	}

	unsafe fn to_repr(self) -> (u64, bool) {
		(self as u64, true)
	}

	fn is_parents(self) -> bool {
		self == Self::__parents__
	}
}
/*
impl Attribute for &'static str {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool> {
		if let Some(text) = rhs.downcast::<Gc<Text>>() {
			Ok(self == text.as_ref()?.as_str())
		} else {
			Ok(false)
		}
	}

	fn try_eq_intern(self, rhs: Intern) -> Result<bool> {
		Ok(self == rhs.as_str())
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

	unsafe fn to_repr(self) -> (u64, bool) {
		(self.to_value().bits(), false)
	}

	fn is_parents(self) -> bool {
		self == "__parents__"
	}
}*/

impl Attribute for AnyValue {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool> {
		AnyValue::try_eq(self, rhs)
	}

	fn try_eq_intern(self, rhs: Intern) -> Result<bool> {
		self.try_eq_value(rhs.as_text().as_any())
	}

	fn try_hash(self) -> Result<u64> {
		AnyValue::try_hash(self)
	}

	fn to_value(self) -> AnyValue {
		self
	}

	unsafe fn to_repr(self) -> (u64, bool) {
		(self.bits(), false)
	}

	fn is_parents(self) -> bool {
		self
			.downcast::<Gc<Text>>()
			.map_or(false, |text| text.as_ref().map_or(false, |r| r.as_str() == "__parents__"))
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
		let text = Text::from_static_str("yo waddup");
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
