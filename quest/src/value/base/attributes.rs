use crate::value::ty::Text;
use crate::value::{base::Flags, Gc, ToValue};
use crate::{Intern, Result, Value};
use std::fmt::{self, Debug, Formatter};
use std::mem::ManuallyDrop;

mod list;
mod map;
use list::ListMap;
use map::Map;

#[repr(C)]
pub(super) union Attributes {
	none: u64,
	list: ManuallyDrop<Box<ListMap>>,
	map: ManuallyDrop<Box<Map>>,
}

sa::assert_eq_size!(Attributes, u64);
sa::assert_eq_align!(Attributes, u64);

/// Immutable access to a [`Header`](crate::value::base::Header)'s attributes.
#[repr(C)]
pub struct AttributesRef<'a> {
	attributes: &'a Attributes,
	flags: &'a Flags,
}

/// Mutable access to a [`Header`](crate::value::base::Header)'s attributes.
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
		AttributesRef { attributes: self, flags }
	}

	pub(super) unsafe fn guard_mut<'a>(&'a mut self, flags: &'a Flags) -> AttributesMut<'a> {
		AttributesMut { attributes: self, flags }
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

	/// Gets an iterator over `self`'s attributes.
	pub fn iter(&self) -> AttributesIter<'_> {
		AttributesIter(if self.is_none() {
			AttributesIterInner::None
		} else if self.isnt_map() {
			AttributesIterInner::List(unsafe { &self.attributes.list }.iter())
		} else {
			AttributesIterInner::Map(unsafe { &self.attributes.map }.iter())
		})
	}

	/// Gets the amount of attributes that were defined.
	pub fn len(&self) -> usize {
		if self.is_none() {
			0
		} else if self.isnt_map() {
			unsafe { &self.attributes.list }.len()
		} else {
			unsafe { &self.attributes.map }.len()
		}
	}

	/// Whether any attributes are defined.
	pub fn is_empty(&self) -> bool {
		// NOTE: WE can't just check for `.is_none()`, as list/map attributes could have been deleted.
		self.len() == 0
	}

	/// Gets an unbound attribute `attr`.
	///
	/// Note that `attr` should not be a [special attribute](Attribute::is_special).
	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		debug_assert!(!attr.is_special());

		if self.is_none() {
			Ok(None)
		} else if self.isnt_map() {
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

	/// Gets mutable access to an unbound attribute `attr`.
	///
	/// Note that `attr` should not be a [special attribute](Attribute::is_special).
	pub fn get_unbound_attr_mut<A: Attribute>(mut self, attr: A) -> Result<&'a mut Value> {
		debug_assert!(!attr.is_special());

		// TODO: don't fetch the attr beforehand
		if self.get_unbound_attr(attr)?.is_none() {
			self.set_attr(attr, Value::default())?;
		}

		debug_assert!(!self.is_none());

		if self.isnt_map() {
			unsafe { &mut self.attributes.list }.get_unbound_attr_mut(attr)
		} else {
			unsafe { &mut self.attributes.map }.get_unbound_attr_mut(attr)
		}
	}

	/// Sets the attribute `attr` to `value`.
	///
	/// Note that `attr` should not be a [special attribute](Attribute::is_special).
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
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

	/// Deletes an attribute `attr`, returning `None` if it didnt exist.
	///
	/// Note that `attr` should not be a [special attribute](Attribute::is_special).
	pub fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
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

/// An iterator over immutable references to attributes.
pub struct AttributesIter<'a>(AttributesIterInner<'a>);

// we need an inner enum so people cant access the internals whilst the iter is public.
enum AttributesIterInner<'a> {
	None,
	List(list::ListMapIter<'a>),
	Map(map::MapIter<'a>),
}

impl Iterator for AttributesIter<'_> {
	type Item = (Value, Value);

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.0 {
			AttributesIterInner::None => None,
			AttributesIterInner::List(list_iter) => list_iter.next(),
			AttributesIterInner::Map(map_iter) => map_iter.next(),
		}
	}
}

/// A helper trait which allows for indexing with more than just `Value`s.
///
/// This may become a sealed trait at some point.
pub trait Attribute: Copy + Debug + ToValue {
	/// See if `self` is equal to the [`Value`] rhs.
	fn try_eq_value(self, rhs: Value) -> Result<bool>;

	/// See if `self` is equal to the [`Intern`] rhs.
	fn try_eq_intern(self, rhs: Intern) -> Result<bool>;

	/// Attempts to convert `self` to an `Intern`.
	fn as_intern(self) -> Result<Option<Intern>>;

	/// Attempt to hash `self`.
	fn try_hash(self) -> Result<u64>;

	/// Get the raw data corresponding to `self`.
	fn bits(self) -> u64;

	/// Checks to see if self is a "special" attribute
	///
	/// Special attributes don't work like normal attributes, and have special hooks associated with
	/// them.
	fn is_special(self) -> bool {
		self.is_parents()
	}

	/// Checks to see if `self` corresponds to [`Intern::__parents__`].
	fn is_parents(self) -> bool;
}

impl Attribute for Intern {
	fn try_eq_value(self, rhs: Value) -> Result<bool> {
		if let Some(text) = rhs.downcast::<Gc<Text>>() {
			Ok(*text.as_ref()? == self)
		} else {
			Ok(false)
		}
	}

	fn try_eq_intern(self, rhs: Self) -> Result<bool> {
		Ok(self == rhs)
	}

	fn try_hash(self) -> Result<u64> {
		Ok(self.fast_hash())
	}

	fn as_intern(self) -> Result<Option<Self>> {
		Ok(Some(self))
	}

	fn bits(self) -> u64 {
		self.bits()
	}

	fn is_parents(self) -> bool {
		self == Self::__parents__
	}
}

impl Attribute for Value {
	fn try_eq_value(self, rhs: Self) -> Result<bool> {
		Self::try_eq(self, rhs)
	}

	fn try_eq_intern(self, rhs: Intern) -> Result<bool> {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			Ok(*text.as_ref()? == rhs)
		} else {
			self.try_eq(rhs.as_text().to_value())
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

	fn bits(self) -> u64 {
		self.bits()
	}

	fn is_parents(self) -> bool {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			*text.as_ref().expect("text is locked <todo, return an error>") == Intern::__parents__
		} else {
			false
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::{Integer, Text};
	use crate::value::{Attributed, AttributedMut, Value};

	#[test]
	fn it_transitions_over_to_full_map() {
		let text = Text::from_static_str("yo waddup");

		{
			let mut textmut = text.as_mut().unwrap();

			for i in 0..=list::MAX_LISTMAP_LEN * 2 {
				let value = Value::from(Integer::new(i as i64).unwrap()).to_value_const();
				textmut.set_attr(value, value).unwrap();

				// assert!(textmut.r().get_unbound_attr(value).unwrap().unwrap().try_eq(value).unwrap());
			}
		}

		let textref = text.as_ref().unwrap();

		let value = Value::from(Integer::new(3).unwrap()).to_value_const();
		assert!(textref
			.get_unbound_attr_checked(value, &mut vec![])
			.unwrap()
			.unwrap()
			.try_eq(value)
			.unwrap());

		// now it should be a full `map`, let's go over all of them again.
		for i in 0..=list::MAX_LISTMAP_LEN * 2 {
			let value = Value::from(Integer::new(i as i64).unwrap()).to_value_const();
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
		const ONE: Value = Value::ONE.to_value_const();

		assert_matches!(text.as_ref().unwrap().get_unbound_attr_checked(ONE, &mut vec![]), Ok(None));
		assert_matches!(text.as_mut().unwrap().del_attr(ONE), Ok(None));

		text
			.as_mut()
			.unwrap()
			.set_attr(ONE, Value::from(Integer::new(23).unwrap()).to_value_const())
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
			Integer::new(23).unwrap()
		);

		text
			.as_mut()
			.unwrap()
			.set_attr(ONE, Value::from(Integer::new(45).unwrap()).to_value_const())
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
			Integer::new(45).unwrap()
		);

		assert_eq!(
			text.as_mut().unwrap().del_attr(ONE).unwrap().unwrap().downcast::<Integer>().unwrap(),
			Integer::new(45).unwrap()
		);
		assert_matches!(text.as_ref().unwrap().get_unbound_attr_checked(ONE, &mut vec![]), Ok(None));
	}

	// XXX: This test may spuriously fail with the message `Message("parents are already locked")`.
	// This is because parent attributes are locked independently of `Base<T>` locking, and all
	// builtin class parent objects (eg `Integer`, etc.) are shared across all tests. So one test
	// may be modifying a parent object whilst the other is trying to read from it, which causes
	// an issue.
	#[test]
	fn parents_work() {
		const ATTR: Value = Value::TRUE.to_value_const();

		let mut parent = Value::from("hello, world").to_value_const();
		parent.set_attr(ATTR, Value::from(Integer::new(123).unwrap()).to_value_const()).unwrap();
		assert_eq!(
			parent
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			Integer::new(123).unwrap()
		);

		let mut child = Value::ONE.to_value_const();
		assert!(!child.has_attr(ATTR).unwrap());

		child.parents_list().unwrap().as_mut().unwrap().push(parent);
		assert_eq!(
			child
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			Integer::new(123).unwrap()
		);

		child.set_attr(ATTR, Value::from(Integer::new(456).unwrap()).to_value_const()).unwrap();
		assert_eq!(
			child
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			Integer::new(456).unwrap()
		);

		assert_eq!(
			child.del_attr(ATTR).unwrap().unwrap().downcast::<Integer>().unwrap(),
			Integer::new(456).unwrap()
		);
		assert_eq!(
			child
				.get_unbound_attr_checked(ATTR, &mut vec![])
				.unwrap()
				.unwrap()
				.downcast::<Integer>()
				.unwrap(),
			Integer::new(123).unwrap()
		);
		assert!(child.del_attr(ATTR).unwrap().is_none()); // cannot delete from parents.
	}
}
