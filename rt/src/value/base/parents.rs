use crate::value::base::{Attribute, Flags};
use crate::value::ty::List;
use crate::value::{AnyValue, Gc};
use crate::{Error, Result};
use std::fmt::{self, Debug, Formatter};
use std::num::NonZeroU64;

#[repr(transparent)]
#[derive(Clone, Copy)]
// Note that this is not a `union` because `sizeof<Option<union { AnyValue, Gc<List> }>>` is not
// eight, which is required for the header. This is a workaround
pub(crate) struct Parents(NonZeroU64);
// pub union Parents {
// 	single: AnyValue,
// 	list: Gc<List>,
// }

pub unsafe trait IntoParent {
	fn into_parent(self, flags: &Flags) -> Option<NonZeroU64>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NoParents;

unsafe impl IntoParent for NoParents {
	#[inline]
	fn into_parent(self, flags: &Flags) -> Option<NonZeroU64> {
		flags.remove_internal(Flags::MULTI_PARENT);
		None
	}
}

unsafe impl IntoParent for AnyValue {
	#[inline]
	fn into_parent(self, flags: &Flags) -> Option<NonZeroU64> {
		flags.remove_internal(Flags::MULTI_PARENT);

		Some(unsafe { std::mem::transmute(self) })
	}
}

unsafe impl IntoParent for Gc<List> {
	#[inline]
	fn into_parent(self, flags: &Flags) -> Option<NonZeroU64> {
		flags.insert_internal(Flags::MULTI_PARENT);

		Some(unsafe { std::mem::transmute(self) })
	}
}

sa::assert_eq_size!(Option<Parents>, u64);
sa::assert_eq_align!(Option<Parents>, u64);

fn is_single(flags: &Flags) -> bool {
	!flags.contains(Flags::MULTI_PARENT)
}

impl Parents {
	pub const fn new(parent: NonZeroU64) -> Self {
		Self(parent)
	}

	/// Gets a debug representation with the given flags.
	///
	/// # Safety
	/// Like all the other functions on `Parents`, `flags` must be from the same `Header`, and
	/// correctly synced.
	pub unsafe fn debug<'a>(self, flags: &'a Flags) -> impl Debug + 'a {
		struct ParentsDebug<'a>(Parents, &'a Flags);
		impl Debug for ParentsDebug<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if is_single(self.1) {
					f.debug_list().entry(unsafe { &self.0.singular() }).finish()
				} else {
					f.debug_list()
						.entries(
							unsafe { &self.0.list() }
								.as_ref()
								.expect("asref failed for entries")
								.as_slice(),
						)
						.finish()
				}
			}
		}

		ParentsDebug(self, flags)
	}

	// SAFETY: we must have a singular parent.
	unsafe fn singular(self) -> AnyValue {
		std::mem::transmute(self)
	}

	// SAFETY: we must have a list of parents.
	unsafe fn list(self) -> Gc<List> {
		std::mem::transmute(self)
	}

	/// Converts `self` into a list of parents if it isn't already, returning the list. Modifications
	/// to the list will modify the the parents.
	///
	/// # Safety
	/// Like all the other functions on `Parents`, `flags` must be from the same `Header`, and
	/// correctly synced.
	pub unsafe fn as_list(&mut self, flags: &Flags) -> Gc<List> {
		if is_single(flags) {
			let list = List::from_slice(&[self.singular()]);
			*self = Self::new(list.into_parent(flags).unwrap());
		}

		self.list()
	}

	/// Attempts to get the unbound attribute `attr` on `self`.
	///
	/// # Safety
	/// Like all the other functions on `Parents`, `flags` must be from the same `Header`, and
	/// correctly synced.
	pub unsafe fn get_unbound_attr<A: Attribute>(
		&self,
		attr: A,
		flags: &Flags,
	) -> Result<Option<AnyValue>> {
		if is_single(flags) {
			return self.singular().get_unbound_attr(attr);
		}

		let list = self.list();

		for parent in list.as_ref()?.as_slice() {
			if let Some(value) = parent.get_unbound_attr(attr)? {
				return Ok(Some(value));
			}
		}

		Ok(None)
	}

	/// Attempts to call the attribute `attr` on `self` with the given args. Note that this is a
	/// distinct function so we can optimize function calls in the future without having to fetch
	/// the attribute first.
	///
	/// # Safety
	/// Like all the other functions on `Parents`, `flags` must be from the same `Header`, and
	/// correctly synced.
	pub unsafe fn call_attr<A: Attribute>(
		&self,
		attr: A,
		args: crate::vm::Args<'_>,
		flags: &Flags,
	) -> Result<AnyValue> {
		self
			.get_unbound_attr(attr, flags)?
			.ok_or_else(|| {
				Error::UnknownAttribute(
					args.get_self().expect("no self given to `call_attr`?"),
					attr.to_value(),
				)
			})?
			.call(args)
	}
}
