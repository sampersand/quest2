use crate::value::base::{Attribute, Flags};
use crate::value::ty::List;
use crate::value::{Gc, Value};
use crate::{ErrorKind, Result};
use std::fmt::{self, Debug, Formatter};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) union Parents {
	none: u64, // will be zero
	single: Value,
	list: Gc<List>,
}

sa::assert_eq_size!(Parents, u64);
sa::assert_eq_align!(Parents, u64);

/// An immutable reference to a [`Header`](crate::value::base::Header)'s parents.
#[repr(C)]
pub struct ParentsRef<'a> {
	parents: &'a Parents,
	flags: &'a Flags,
}

/// A mutable reference to a [`Header`](crate::value::base::Header)'s parents.
#[repr(C)]
pub struct ParentsMut<'a> {
	parents: &'a mut Parents,
	flags: &'a Flags,
}

sa::assert_eq_size!(ParentsRef<'_>, ParentsMut<'_>);
sa::assert_eq_align!(ParentsRef<'_>, ParentsMut<'_>);

impl<'a> std::ops::Deref for ParentsMut<'a> {
	type Target = ParentsRef<'a>;

	fn deref(&self) -> &Self::Target {
		unsafe { std::mem::transmute(self) }
	}
}

impl Parents {
	pub(super) unsafe fn guard_ref<'a>(&'a self, flags: &'a Flags) -> ParentsRef<'a> {
		ParentsRef { parents: self, flags }
	}

	pub(super) unsafe fn guard_mut<'a>(&'a mut self, flags: &'a Flags) -> ParentsMut<'a> {
		ParentsMut { parents: self, flags }
	}
}

/// A trait that indicates a type can be converted into a [`Header](crate::value::base::Header)'s
/// parents.
pub trait IntoParent {
	/// Replaces `parents` with `self`.
	fn into_parent(self, parents: &mut ParentsMut<'_>);
}

/// Indicates that no parents are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoParents;

impl IntoParent for NoParents {
	#[inline]
	fn into_parent(self, parents: &mut ParentsMut<'_>) {
		parents.flags.remove_internal(Flags::MULTI_PARENT);
		parents.parents.none = 0;
	}
}

impl IntoParent for Value {
	#[inline]
	fn into_parent(self, parents: &mut ParentsMut<'_>) {
		parents.flags.remove_internal(Flags::MULTI_PARENT);
		parents.parents.single = self;
	}
}

impl IntoParent for Gc<List> {
	#[inline]
	fn into_parent(self, parents: &mut ParentsMut<'_>) {
		parents.flags.insert_internal(Flags::MULTI_PARENT);
		parents.parents.list = self;
	}
}

impl IntoParent for ParentsRef<'_> {
	fn into_parent(self, parents: &mut ParentsMut<'_>) {
		match self.classify() {
			ParentsKind::None => NoParents.into_parent(parents),
			ParentsKind::Single(single) => single.into_parent(parents),
			ParentsKind::List(list) => list.as_ref().unwrap().dup().into_parent(parents),
		}
	}
}

enum ParentsKind {
	None,
	Single(Value),
	List(Gc<List>),
}

impl ParentsRef<'_> {
	#[cfg(debug_assertions)]
	pub(crate) fn _is_just_single_and_identical(&self, what: Value) -> bool {
		if let ParentsKind::Single(single) = self.classify() {
			single.is_identical(what)
		} else {
			false
		}
	}

	fn classify(&self) -> ParentsKind {
		unsafe {
			if self.parents.none == 0 {
				ParentsKind::None
			} else if !self.flags.contains(Flags::MULTI_PARENT) {
				ParentsKind::Single(self.parents.single)
			} else {
				ParentsKind::List(self.parents.list)
			}
		}
	}

	/// Attempts to get the unbound attribute `attr` on `self`.
	pub fn get_unbound_attr_checked<A: Attribute>(
		&self,
		attr: A,
		checked: &mut Vec<Value>,
	) -> Result<Option<Value>> {
		match self.classify() {
			ParentsKind::None => Ok(None),
			ParentsKind::Single(single) => single.get_unbound_attr_checked(attr, checked),
			ParentsKind::List(list) => {
				for parent in list.as_ref()?.as_slice() {
					if let Some(value) = parent.get_unbound_attr_checked(attr, checked)? {
						return Ok(Some(value));
					}
				}

				Ok(None)
			}
		}
	}

	/// Attempts to call the attribute `attr` on `self` with the given args. Note that this is a
	/// distinct function so we can optimize function calls in the future without having to fetch
	/// the attribute first.
	pub fn call_attr<A: Attribute>(
		&self,
		obj: Value,
		attr: A,
		args: crate::vm::Args<'_>,
	) -> Result<Value> {
		let attr = self
			.get_unbound_attr_checked(attr, &mut Vec::new())?
			.ok_or_else(|| ErrorKind::UnknownAttribute { object: obj, attribute: attr.to_value() })?;

		drop(self);

		attr.call(args.with_this(obj))
	}
}

impl ParentsMut<'_> {
	/// Replaces `self` with `parent`.
	pub fn set<I: IntoParent>(&mut self, parent: I) {
		parent.into_parent(self);
	}

	/// Converts `self` into a list of parents if it isn't already, returning the list. Modifications
	/// to the list will modify the the parents.
	pub fn as_list(&mut self) -> Gc<List> {
		match self.classify() {
			ParentsKind::None => {
				let list = List::new();
				self.set(list);
				list
			}
			ParentsKind::Single(singular) => {
				let list = List::from_slice(&[singular]);
				self.set(list);
				list
			}
			ParentsKind::List(list) => list,
		}
	}
}

impl Debug for ParentsRef<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut l = f.debug_list();

		match self.classify() {
			ParentsKind::None => {}
			ParentsKind::Single(s) => {
				l.entry(&s);
			}
			ParentsKind::List(s) => {
				l.entries(s.as_ref().expect("asref failed for entries").as_slice());
			}
		};

		l.finish()
	}
}

impl Debug for ParentsMut<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		<ParentsRef as Debug>::fmt(self, f)
	}
}
