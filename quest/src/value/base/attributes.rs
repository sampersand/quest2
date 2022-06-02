use crate::value::ty::Text;
use crate::value::{base::Flags, Gc, Intern, ToAny};
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

#[repr(C)]
pub struct AttributesRef<'a> {
	attributes: &'a Attributes,
	flags: &'a Flags,
}

#[repr(C)]
pub struct AttributesMut<'a> {
	attributes: &'a mut Attributes,
	flags: &'a Flags,
}

sa::assert_eq_size!(AttributesRef<'_>, AttributesMut<'_>);
sa::assert_eq_align!(AttributesRef<'_>, AttributesMut<'_>);

impl<'a> std::ops::Deref for AttributesMut<'a> {
	type Target = AttributesRef<'a>;

	fn deref(&self) -> &Self::Target {
		unsafe { std::mem::transmute(self) }
	}
}

impl Attributes {
	pub(super) unsafe fn guard_ref<'a>(&'a self, flags: &'a Flags) -> AttributesRef<'a> {
		AttributesRef {
			attributes: self,
			flags,
		}
	}

	pub(super) unsafe fn guard_mut<'a>(&'a mut self, flags: &'a Flags) -> AttributesMut<'a> {
		AttributesMut {
			attributes: self,
			flags,
		}
	}
}

impl Debug for AttributesRef<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.is_none() {
			f.debug_map().finish()
		} else if self.isnt_map() {
			Debug::fmt(unsafe { &self.attributes.list }, f)
		} else {
			Debug::fmt(unsafe { &self.attributes.map }, f)
		}
	}
}

impl Debug for AttributesMut<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		<AttributesRef as Debug>::fmt(self, f)
	}
}

impl<'a> AttributesRef<'a> {
	fn is_none(&self) -> bool {
		unsafe { self.attributes.none == 0 }
	}

	fn isnt_map(&self) -> bool {
		!self.flags.contains(Flags::ATTR_MAP)
	}

	pub fn iter(&self) -> AttributesIter<'_> {
		AttributesIter(if self.is_none() {
			AttributesIterInner::None
		} else if self.isnt_map() {
			AttributesIterInner::List(unsafe { &self.attributes.list }.iter())
		} else {
			AttributesIterInner::Map(unsafe { &self.attributes.map }.iter())
		})
	}

	pub fn len(&self) -> usize {
		if self.is_none() {
			0
		} else if self.isnt_map() {
			unsafe { &self.attributes.list }.len()
		} else {
			unsafe { &self.attributes.map }.len()
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		if self.is_none() {
			return Ok(None);
		}

		if self.isnt_map() {
			unsafe { &self.attributes.list }.get_unbound_attr(attr)
		} else {
			unsafe { &self.attributes.map }.get_unbound_attr(attr)
		}
	}
}

impl<'a> AttributesMut<'a> {
	pub(crate) fn allocate(&mut self, capacity: usize) {
		self.flags.remove_internal(Flags::ATTR_MAP);

		if capacity == 0 {
			self.attributes.none = 0;
		} else if capacity <= list::MAX_LISTMAP_LEN {
			self.attributes.list = ManuallyDrop::new(ListMap::new());
		} else {
			assert!(
				capacity <= isize::MAX as usize,
				"can only allocate up to isize::MAX ({capacity} is too big)"
			);

			self.flags.insert_internal(Flags::ATTR_MAP);
			self.attributes.map = ManuallyDrop::new(Map::with_capacity(capacity));
		}
	}

	pub fn get_unbound_attr_mut<A: Attribute>(mut self, attr: A) -> Result<&'a mut AnyValue> {
		debug_assert!(!attr.is_special());

		// TODO: don't fetch the attr beforehand
		if self.get_unbound_attr(attr)?.is_none() {
			self.set_attr(attr, AnyValue::default())?;
		}

		debug_assert!(!self.is_none());

		if self.isnt_map() {
			unsafe { &mut self.attributes.list }.get_unbound_attr_mut(attr)
		} else {
			unsafe { &mut self.attributes.map }.get_unbound_attr_mut(attr)
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		debug_assert!(!attr.is_special());

		if self.is_none() {
			debug_assert!(self.isnt_map());

			self.attributes.list = ManuallyDrop::new(ListMap::new());
			return unsafe { &mut self.attributes.list }.set_attr(attr, value);
		}

		if self.isnt_map() {
			let list = unsafe { &mut self.attributes.list };
			if !list.is_full() {
				return list.set_attr(attr, value);
			}

			let list = unsafe { ManuallyDrop::take(list) };
			self.attributes.map = ManuallyDrop::new(Map::from_iter(list.iter())?);
			self.flags.insert_internal(Flags::ATTR_MAP);
		}

		unsafe { &mut self.attributes.map }.set_attr(attr, value)
	}

	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<AnyValue>> {
		debug_assert!(!attr.is_special());

		if self.is_none() {
			Ok(None)
		} else if self.isnt_map() {
			unsafe { &mut self.attributes.list }.del_attr(attr)
		} else {
			unsafe { &mut self.attributes.map }.del_attr(attr)
		}
	}

	pub(crate) unsafe fn drop_internal(&mut self) {
		if self.is_none() {
			// we do nothing when dropping empty attributes
		} else if self.isnt_map() {
			ManuallyDrop::drop(&mut self.attributes.list)
		} else {
			ManuallyDrop::drop(&mut self.attributes.map)
		}
	}
}

pub struct AttributesIter<'a>(AttributesIterInner<'a>);

// we need an inner enum so people cant access the internals whilst the iter is public.
enum AttributesIterInner<'a> {
	None,
	List(list::ListMapIter<'a>),
	Map(map::MapIter<'a>),
}

impl Iterator for AttributesIter<'_> {
	type Item = (AnyValue, AnyValue);

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.0 {
			AttributesIterInner::None => None,
			AttributesIterInner::List(list_iter) => list_iter.next(),
			AttributesIterInner::Map(map_iter) => map_iter.next(),
		}
	}
}

pub trait Attribute: Copy + Debug {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool>;
	fn try_eq_intern(self, rhs: Intern) -> Result<bool>;

	fn as_intern(self) -> Result<Option<Intern>>;

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
		Ok(self.fast_hash())
	}

	fn as_intern(self) -> Result<Option<Intern>> {
		Ok(Some(self))
	}

	fn to_value(self) -> AnyValue {
		self.as_text().to_any()
	}

	fn to_repr(self) -> u64 {
		self as u64
	}

	fn is_parents(self) -> bool {
		self == Self::__parents__
	}
}

impl Attribute for AnyValue {
	fn try_eq_value(self, rhs: AnyValue) -> Result<bool> {
		Self::try_eq(self, rhs)
	}

	fn try_eq_intern(self, rhs: Intern) -> Result<bool> {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			Ok(*text.as_ref()? == rhs)
		} else {
			self.try_eq(rhs.as_text().to_any())
		}
	}

	fn try_hash(self) -> Result<u64> {
		Self::try_hash(self)
	}

	fn as_intern(self) -> Result<Option<Intern>> {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			Ok(Intern::try_from(&*text.as_ref()?).ok())
		} else {
			Ok(None)
		}
	}

	fn to_value(self) -> AnyValue {
		self
	}

	fn to_repr(self) -> u64 {
		self.bits()
	}

	fn is_parents(self) -> bool {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			*text
				.as_ref()
				.expect("text is locked <todo, return an error>")
				== Intern::__parents__
		} else {
			false
		}
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

		{
			let mut textmut = text.as_mut().unwrap();

			for i in 0..=list::MAX_LISTMAP_LEN * 2 {
				let value = Value::from(i as i64).any();
				textmut.set_attr(value, value).unwrap();

				// assert!(textmut.r().get_unbound_attr(value).unwrap().unwrap().try_eq(value).unwrap());
			}
		}

		let textref = text.as_ref().unwrap();

		let value = Value::from(3).any();
		assert!(textref
			.get_unbound_attr_checked(value, &mut vec![])
			.unwrap()
			.unwrap()
			.try_eq(value)
			.unwrap());

		// now it should be a full `map`, let's go over all of them again.
		for i in 0..=list::MAX_LISTMAP_LEN * 2 {
			let value = Value::from(i as i64).any();
			assert!(textref
				.get_unbound_attr_checked(value, &mut vec![])
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

		assert_matches!(
			text
				.as_ref()
				.unwrap()
				.get_unbound_attr_checked(ONE, &mut vec![]),
			Ok(None)
		);
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
				.get_unbound_attr_checked(ONE, &mut vec![])
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
				.get_unbound_attr_checked(ONE, &mut vec![])
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
		assert_matches!(
			text
				.as_ref()
				.unwrap()
				.get_unbound_attr_checked(ONE, &mut vec![]),
			Ok(None)
		);
	}

	// XXX: This test may spuriously fail with the message `Message("parents are already locked")`.
	// This is because parent attributes are locked independently of `Base<T>` locking, and all
	// builtin class parent objects (eg `Integer`, etc.) are shared across all tests. So one test
	// may be modifying a parent object whilst the other is trying to read from it, which causes
	// an issue.
	#[test]
	fn parents_work() {
		const ATTR: AnyValue = Value::TRUE.any();

		let mut parent = Value::from("hello, world").any();
		parent.set_attr(ATTR, Value::from(123).any()).unwrap();
		assert_eq!(
			parent
				.get_unbound_attr_checked(ATTR, &mut vec![])
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
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			123
		);

		child.set_attr(ATTR, Value::from(456).any()).unwrap();
		assert_eq!(
			child
				.get_unbound_attr_checked(ATTR, &mut vec![])
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
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			123
		);
		assert!(child.del_attr(ATTR).unwrap().is_none()); // cannot delete from parents.
	}
}
