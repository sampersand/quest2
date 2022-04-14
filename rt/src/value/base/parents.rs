use crate::value::base::{Attribute, Flags};
use crate::value::ty::List;
use crate::value::{AnyValue, Gc};
use crate::{Error, Result};
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

pub struct ParentsGuard<'a> {
	ptr: *mut Parents,
	flags: &'a Flags
}

impl Drop for ParentsGuard<'_> {
	fn drop(&mut self) {
		let remove = self.flags.remove_internal(Flags::LOCK_PARENTS);
		debug_assert!(remove, "couldn't remove parents lock?");
	}
}

pub unsafe trait IntoParent {
	fn into_parent(self, guard: &mut ParentsGuard<'_>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoParents;

unsafe impl IntoParent for NoParents {
	#[inline]
	fn into_parent(self, guard: &mut ParentsGuard<'_>) {
		guard.flags.remove_internal(Flags::MULTI_PARENT);
		unsafe { guard.ptr.write(Parents { none: 0 }); }
	}
}

unsafe impl IntoParent for AnyValue {
	#[inline]
	fn into_parent(self, guard: &mut ParentsGuard<'_>) {
		guard.flags.remove_internal(Flags::MULTI_PARENT);
		unsafe { guard.ptr.write(Parents { single: self }); }
	}
}

unsafe impl IntoParent for Gc<List> {
	#[inline]
	fn into_parent(self, guard: &mut ParentsGuard<'_>) {
		guard.flags.insert_internal(Flags::MULTI_PARENT);
		unsafe { guard.ptr.write(Parents { list: self }); }
	}
}

enum ParentsKind {
	None,
	Single(*mut AnyValue),
	List(*mut Gc<List>)
}

impl<'a> ParentsGuard<'a> {
	// safety: parents and flags have to correspond
	pub(super) unsafe fn new(ptr: *mut Parents, flags: &'a Flags) -> Option<Self> {
		if flags.try_acquire_all_internal(Flags::LOCK_PARENTS) {
			Some(Self { ptr, flags })
		} else {
			None
		}
	}

	pub(crate) fn set<I: IntoParent>(&mut self, parent: I) {
		parent.into_parent(self);
	}

	fn classify(&self) -> ParentsKind {
		unsafe {
			if (*self.ptr).none == 0 {
				ParentsKind::None
			} else if !self.flags.contains(Flags::MULTI_PARENT) {
				ParentsKind::Single(self.ptr.cast::<AnyValue>())
			} else {
				ParentsKind::List(self.ptr.cast::<Gc<List>>())
			}
		}
	}
}

impl Debug for ParentsGuard<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut l = f.debug_list();
		match self.classify() {
			ParentsKind::None => {},
			ParentsKind::Single(s) => { l.entry(unsafe { &*s }); },
			ParentsKind::List(s) => {
				l.entries(unsafe { *s }.as_ref().expect("asref failed for entries").as_slice());
			},
		};
		l.finish()
	}
}

impl ParentsGuard<'_> {
	/// Converts `self` into a list of parents if it isn't already, returning the list. Modifications
	/// to the list will modify the the parents.
	pub fn as_list(&mut self) -> Gc<List> {
		let list;
		match self.classify() {
			ParentsKind::None => {
				list = List::new();
				self.set(list);
			},
			ParentsKind::Single(singular) => {
				list = List::from_slice(&[unsafe { *singular }]);
				self.set(list);
			},
			ParentsKind::List(list_) => list = unsafe { *list_ }
		}
		list
	}

	/// Attempts to get the unbound attribute `attr` on `self`.
	pub fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<AnyValue>> {
		match self.classify() {
			ParentsKind::None => Ok(None),
			ParentsKind::Single(single) => unsafe { *single }.get_unbound_attr(attr),
			ParentsKind::List(list) => {
				for parent in unsafe { *list }.as_ref()?.as_slice() {
					if let Some(value) = parent.get_unbound_attr(attr)? {
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
	pub fn call_attr<A: Attribute>(self, attr: A, args: crate::vm::Args<'_>) -> Result<AnyValue> {
		let attr = self.get_unbound_attr(attr)?
			.ok_or_else(|| {
				Error::UnknownAttribute(
					args.get_self().expect("no self given to `call_attr`?"),
					attr.to_value(),
				)
			})?;
		drop(self);
		attr.call(args)
	}
}
