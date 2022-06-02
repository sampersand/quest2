use crate::value::base::{Attribute, Flags};
use crate::value::ty::List;
use crate::value::{AnyValue, Gc};
use crate::Result;
use std::fmt::{self, Debug, Formatter};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) union Parents {
	none: u64, // will be zero
	single: AnyValue,
	list: Gc<List>,
}

sa::assert_eq_size!(Parents, u64);
sa::assert_eq_align!(Parents, u64);

#[repr(C)]
pub struct ParentsRef<'a> {
	parents: &'a Parents,
	flags: &'a Flags,
}

#[repr(C)]
pub struct ParentsMut<'a> {
	parents: &'a mut Parents,
	flags:  &'a Flags,
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

pub unsafe trait IntoParent {
	fn into_parent(self, guard: &mut ParentsMut<'_>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoParents;

unsafe impl IntoParent for NoParents {
	#[inline]
	fn into_parent(self, guard: &mut ParentsMut<'_>) {
		guard.flags.remove_internal(Flags::MULTI_PARENT);
		guard.parents.none = 0;
	}
}

unsafe impl IntoParent for AnyValue {
	#[inline]
	fn into_parent(self, guard: &mut ParentsMut<'_>) {
		guard.flags.remove_internal(Flags::MULTI_PARENT);
		guard.parents.single = self;
	}
}

unsafe impl IntoParent for Gc<List> {
	#[inline]
	fn into_parent(self, guard: &mut ParentsMut<'_>) {
		guard.flags.insert_internal(Flags::MULTI_PARENT);
		guard.parents.list = self;
	}
}

unsafe impl IntoParent for ParentsRef<'_> {
	fn into_parent(self, guard: &mut ParentsMut<'_>) {
		match self.classify() {
			ParentsKind::None => NoParents.into_parent(guard),
			ParentsKind::Single(single) => single.into_parent(guard),
			ParentsKind::List(list) => list.as_ref().unwrap().dup().into_parent(guard),
		}
	}
}

enum ParentsKind {
	None,
	Single(AnyValue),
	List(Gc<List>),
}

impl<'a> ParentsRef<'a> {
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
	pub fn get_unbound_attr_checked<A: Attribute>(&self, attr: A, checked: &mut Vec<AnyValue>) -> Result<Option<AnyValue>> {
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
	// TODO: we should take by-reference, but this solves an issue with gc until we make gc only for body.
	pub fn call_attr<A: Attribute>(
		self,
		obj: AnyValue,
		attr: A,
		args: crate::vm::Args<'_>,
	) -> Result<AnyValue> {
		let attr = self
			.get_unbound_attr_checked(attr, &mut Vec::new())?
			.ok_or_else(|| crate::error::ErrorKind::UnknownAttribute {
				object: obj,
				attribute: attr.to_value()
			})?;

		drop(self);

		attr.call(args.with_self(obj))
	}
}

impl ParentsMut<'_> {
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
			},
			ParentsKind::Single(singular) => {
				let list = List::from_slice(&[singular]);
				self.set(list);
				list
			},
			ParentsKind::List(list) => list
		}
	}
}


impl Debug for ParentsRef<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut l = f.debug_list();

		match self.classify() {
			ParentsKind::None => {},
			ParentsKind::Single(s) => { l.entry(&s); },
			ParentsKind::List(s) => {
				l.entries(
					s
						.as_ref()
						.expect("asref failed for entries")
						.as_slice(),
				);
			},
		};

		l.finish()
	}
}

impl Debug for ParentsMut<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		<ParentsRef as Debug>::fmt(self, f)
	}
}
