use crate::value::ty::Text;
use crate::value::{base::Flags, AsAny, Gc, Intern};
use crate::{AnyValue, Result};
use std::fmt::{self, Debug, Formatter};
use std::mem::ManuallyDrop;

mod internkey;
mod list;
mod map;
use internkey::InternKey;
use list::ListMap;
use map::Map;

#[repr(C)]
pub union Attributes {
	none: u64,
	list: ManuallyDrop<Box<ListMap>>,
	map: ManuallyDrop<Box<Map>>,
}

sa::assert_eq_size!(Attributes, u64);
sa::assert_eq_align!(Attributes, u64);

pub struct AttributesGuard<'a> {
	ptr: *mut Attributes,
	flags: &'a Flags,
}

impl Drop for AttributesGuard<'_> {
	fn drop(&mut self) {
		let remove = self.flags.remove_internal(Flags::LOCK_ATTRIBUTES);
		debug_assert!(remove, "couldn't remove attributes lock?");
	}
}

impl Debug for AttributesGuard<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self.classify() {
			AttributesKind::None => f.debug_map().finish(),
			AttributesKind::List(list) => Debug::fmt(unsafe { &*list }, f),
			AttributesKind::Map(map) => Debug::fmt(unsafe { &*map }, f),
		}
	}
}

enum AttributesKind {
	None,
	List(*mut ListMap),
	Map(*mut Map),
}

impl<'a> AttributesGuard<'a> {
	pub(super) unsafe fn new(ptr: *mut Attributes, flags: &'a Flags) -> Option<Self> {
		if flags.try_acquire_all_internal(Flags::LOCK_ATTRIBUTES) {
			Some(Self { ptr, flags })
		} else {
			None
		}
	}

	pub fn allocate(&mut self, capacity: usize) {
		match capacity {
			0 => {
				self.flags.remove_internal(Flags::ATTR_MAP);
				unsafe {
					self.ptr.write(Attributes { none: 0 });
				}
			},
			1..=list::MAX_LISTMAP_LEN => {
				self.flags.remove_internal(Flags::ATTR_MAP);
				unsafe {
					self.ptr.write(Attributes {
						list: ManuallyDrop::new(ListMap::default().into()),
					});
				}
			},
			other if other < isize::MAX as usize => {
				self.flags.insert_internal(Flags::ATTR_MAP);
				unsafe {
					self.ptr.write(Attributes {
						map: ManuallyDrop::new(Map::with_capacity(capacity).into()),
					})
				}
			},
			other => panic!("can only allocate up to isize::MAX ({} is too big)", other),
		}
	}

	fn classify(&self) -> AttributesKind {
		if unsafe { (*self.ptr).none } == 0 {
			// since we only go to an attr map once we have enough elements, and never go back,
			// we shouldnt have this ever set here.
			debug_assert!(!self.flags.contains(Flags::ATTR_MAP));

			AttributesKind::None
		} else if !self.flags.contains(Flags::ATTR_MAP) {
			AttributesKind::List(unsafe { &mut **(*self.ptr).list as *mut ListMap })
		} else {
			AttributesKind::Map(unsafe { &mut **(*self.ptr).map as *mut Map })
		}
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		match self.classify() {
			AttributesKind::None => Ok(None),
			AttributesKind::List(list) => unsafe { &*list }.get_unbound_attr(attr),
			AttributesKind::Map(map) => unsafe { &*map }.get_unbound_attr(attr),
		}
	}

	pub fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&'a mut AnyValue> {
		debug_assert!(!attr.is_special());

		if self.get_unbound_attr(attr)?.is_none() {
			self.set_attr(attr, Default::default())?;
		}

		match self.classify() {
			AttributesKind::None => unreachable!("we just set it"),
			AttributesKind::List(list) => unsafe { &mut *list }.get_unbound_attr_mut(attr),
			AttributesKind::Map(map) => unsafe { &mut *map }.get_unbound_attr_mut(attr),
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		debug_assert!(!attr.is_special());

		match self.classify() {
			AttributesKind::None => unsafe {
				self.allocate(1); // we have at least one element
				return (&mut *(*self.ptr).list).set_attr(attr, value);
			},
			AttributesKind::List(list) => unsafe {
				if (*list).is_full() {
					list.cast::<Map>().write(Map::from_iter(
						ManuallyDrop::take(&mut *list.cast::<ManuallyDrop<ListMap>>()).iter(),
					)?);
					self.flags.insert_internal(Flags::ATTR_MAP);
				// we're now a map
				} else {
					return (*list).set_attr(attr, value);
				}
			},
			AttributesKind::Map(_) => {},
		}

		unsafe { &mut *(*self.ptr).map }.set_attr(attr, value)
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		match self.classify() {
			AttributesKind::None => Ok(None),
			AttributesKind::List(list) => unsafe { &mut *list }.del_attr(attr),
			AttributesKind::Map(map) => unsafe { &mut *map }.del_attr(attr),
		}
	}

	pub(crate) unsafe fn drop_internal(&mut self) {
		match self.classify() {
			AttributesKind::None => {},
			AttributesKind::List(list) => ManuallyDrop::<ListMap>::drop(&mut *list.cast()),
			AttributesKind::Map(map) => ManuallyDrop::<Map>::drop(&mut *map.cast()),
		}
	}
}

pub trait Attribute: Copy + Debug {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool>;
	fn try_eq_intern(self, rhs: Intern) -> Result<bool>;

	fn as_intern(self) -> Option<Intern> {
		None
	}

	fn try_hash(self) -> Result<u64>;
	fn to_value(self) -> AnyValue;
	fn to_repr(self) -> u64;

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

	fn as_intern(self) -> Option<Intern> {
		Some(self)
	}

	fn to_value(self) -> AnyValue {
		self.as_text().as_any()
	}

	fn to_repr(self) -> u64 {
		self as u64
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

	fn to_repr(self) -> (u64, bool) {
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

	fn to_repr(self) -> u64 {
		self.bits()
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
