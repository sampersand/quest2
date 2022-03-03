use crate::value::gc::Allocated;
use crate::value::ty::{List, Wrap};
use crate::value::value::Any;
use crate::value::{AnyValue, Gc, Value};
use crate::Result;

/*
000...000 000 = none (ie `Pristine`)
XXX...XXX XX0 = singular parent (nonzero `X`)
XXX...XXX XX1 = Gc<List> (remove `1` before interacting with it)
*/
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Parents(u64);

impl Parents {
	const fn is_empty(&self) -> bool {
		self.0 == 0
	}

	// You can only have allocated values as parents. Unallocated values have to be boxed before
	// they can become parents. (but this isn't a very common occurrence so it seems fine.)
	pub fn new_singular<T: Allocated>(parent: Gc<T>) -> Self {
		let bits = parent.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 1, 0);
		Self(bits)
	}

	pub fn new_list(list: Gc<List>) -> Self {
		let bits = list.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 1, 0);
		Self(bits | 1)
	}

	unsafe fn as_singular_unchecked(&self) -> Gc<Wrap<Any>> {
		debug_assert!(!self.is_empty());
		debug_assert_eq!(self.0 & 1, 0);

		Gc::new_unchecked(self.0 as *mut _)
	}

	unsafe fn as_list_unchecked(&self) -> Gc<List> {
		debug_assert_eq!(self.0 & 1, 1);

		Gc::new_unchecked((self.0 - 1) as usize as *mut _)
	}

	pub fn as_list(&mut self) -> Gc<List> {
		if self.is_empty() {
			*self = Self::new_list(Gc::default());
		} else if self.0 & 1 == 0 {
			let parent = Value::from(unsafe { self.as_singular_unchecked() }).any();
			*self = Self::new_list(List::from_slice(&[parent]));
		}

		debug_assert_eq!(self.0 & 1, 1);

		unsafe { self.as_list_unchecked() }
	}
}

impl Parents {
	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		if self.is_empty() {
			return Ok(None);
		}

		if self.0 & 1 == 0 {
			return unsafe { self.as_singular_unchecked() }
				.as_ref()?
				.get_attr(attr);
		}

		let list = unsafe { self.as_list_unchecked() };

		for parent in list.as_ref()?.as_slice() {
			if let Some(value) = parent.get_attr(attr)? {
				return Ok(Some(value));
			}
		}

		Ok(None)
	}
}
