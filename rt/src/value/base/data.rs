use super::Flags;
use std::sync::atomic::{AtomicU32, Ordering};
use std::ops::{Deref, DerefMut};

pub struct DataRefGuard<'a, T> {
	ptr: *const T,
	#[allow(unused)] // later they'll be used to prevent writer starvation
	flags: &'a Flags,
	borrows: &'a AtomicU32,
}

pub struct DataMutGuard<'a, T> {
	ptr: *mut T,
	#[allow(unused)] // later they'll be used to prevent writer starvation
	flags: &'a Flags,
	borrows: &'a AtomicU32,
}


const MUT_BORROW: u32 = u32::MAX;
pub const MAX_BORROWS: usize = (MUT_BORROW - 1) as usize;

impl<'a, T> DataRefGuard<'a, T> {
	pub(super) fn new(ptr: *const T, flags: &'a Flags, borrows: &'a AtomicU32) -> Option<Self> {
		fn updatefn(x: u32) -> Option<u32> {
			if x == MUT_BORROW {
				None
			} else {
				Some(x + 1)
			}
		}

		match borrows.fetch_update(Ordering::Acquire, Ordering::Relaxed, updatefn) {
			Ok(x) if x == MAX_BORROWS as u32 => panic!("too many immutable borrows"),
			Ok(_) => Some(Self { ptr, flags, borrows }),
			Err(_) => None,
		}
	}
}

impl<T> Drop for DataRefGuard<'_, T> {
	fn drop(&mut self) {
		let prev = self.borrows.fetch_sub(1, Ordering::Release);

		// Sanity check, as it's impossible for us to have a `MUT_BORROW` after a `Ref` is created.
		debug_assert_ne!(prev, MUT_BORROW);

		// Another sanity check, as this indicates something double freed (or a `Mut` was
		// incorrectly created).
		debug_assert_ne!(prev, 0);
	}
}

impl<T> Deref for DataRefGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.ptr }
	}
}


impl<'a, T> DataMutGuard<'a, T> {
	pub(super) fn new(ptr: *mut T, flags: &'a Flags, borrows: &'a AtomicU32) -> Option<Self> {
		if borrows.compare_exchange(0, MUT_BORROW, Ordering::Acquire, Ordering::Relaxed).is_err(){
			None
		} else {
			Some(Self { ptr, flags, borrows })
		}
	}
}

impl<T> Drop for DataMutGuard<'_, T> {
	fn drop(&mut self) {
		if cfg!(debug_assertions) {
			// Sanity check to ensure that the value was previously `MUT_BORROW`
			debug_assert_eq!(MUT_BORROW, self.borrows.swap(0, Ordering::Release));
		} else {
			self.borrows.store(0, Ordering::Release);
		}
	}
}

impl<T> Deref for DataMutGuard<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.ptr }
	}
}

impl<T> DerefMut for DataMutGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.ptr }
	}
}

