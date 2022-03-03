use crate::value::ty::List;
use crate::value::base::Flags;
use crate::value::{AnyValue, Gc, Value};
use std::fmt::{self, Debug, Formatter};
use crate::Result;

#[repr(C)]
#[derive(Clone, Copy)]
pub union Parents {
	none: u64,
	single: AnyValue,
	list: Gc<List>
}

sa::assert_eq_size!(Parents, u64);
sa::assert_eq_align!(Parents, u64);

impl Default for Parents {
	fn default() -> Self {
		Self { none: 0 }
	}
}

fn is_single(flags: &Flags) -> bool {
	flags.contains(Flags::SINGLE_PARENT)
}

impl Parents {
	pub fn debug<'a>(self, flags: &'a Flags) -> impl Debug + 'a {
		struct ParentsDebug<'a>(Parents, &'a Flags);
		impl Debug for ParentsDebug<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if self.0.is_none() {
					f.debug_tuple("None").finish()
				} else if is_single(self.1) {
					f.debug_tuple("Single").field(unsafe { &self.0.single }).finish()
				} else {
					f.debug_tuple("List").field(unsafe { &self.0.list }).finish()
				}
			}
		}

		ParentsDebug(self, flags)
	}

	const fn is_none(self) -> bool {
		unsafe {
			self.none == 0
		}
	}

	pub fn new_singular(parent: AnyValue) -> Self {
		Self { single: parent }
	}

	pub fn new_list(list: Gc<List>) -> Self {
		Self { list }
	}

	pub fn as_list(&mut self, flags: &Flags) -> Gc<List> {
		if self.is_none() {
			self.list = Gc::default();
		} else if is_single(flags) {
			let parent = Value::from(unsafe { self.single }).any();
			self.list = List::from_slice(&[parent]);
			flags.remove(Flags::SINGLE_PARENT);
		}

		unsafe {
			self.list
		}
	}
}

impl Parents {
	pub fn get_attr(&self, attr: AnyValue, flags: &Flags) -> Result<Option<AnyValue>> {
		if self.is_none() {
			return Ok(None);
		}

		if is_single(flags) {
			return unsafe { self.single }.get_attr(attr);
		}

		let list = unsafe { self.list };

		for parent in list.as_ref()?.as_slice() {
			if let Some(value) = parent.get_attr(attr)? {
				return Ok(Some(value));
			}
		}

		Ok(None)
	}
}
