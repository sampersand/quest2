use std::ptr::NonNull;
use crate::{Allocated, base::BaseFlags};
use std::ops::{Deref, DerefMut};
use std::fmt::{self, Debug, Display, Formatter};

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Gc<T: 'static>(NonNull<T>);

impl<T: 'static> Copy for Gc<T> {}
impl<T: 'static> Clone for Gc<T> {
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

#[derive(Debug)]
pub struct AlreadyLockedError {
	_priv: ()
}

impl Display for AlreadyLockedError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("the value is already being operated on by something else.")
	}
}

impl std::error::Error for AlreadyLockedError {}

impl<T: 'static> Gc<T> {
	#[inline]
	pub(crate) unsafe fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}

	pub unsafe fn as_ptr(self) -> *const T {
		self.0.as_ptr() as *const T
	}

	pub unsafe fn as_mut_ptr_unchecked(self) -> *mut T {
		self.0.as_ptr()
	}

	#[inline]
	pub fn flags(&self) -> &BaseFlags {
		self.upcast().flags()
	}

	#[inline]
	pub(crate) fn upcast(&self) -> &Allocated<T> {
		unsafe {
			&*Allocated::upcast(self.as_ptr())
		}
	}

	pub fn as_ref(&self) -> crate::Result<GcRef<'_, T>> {
		let allocated = self.upcast();

		if allocated.flags().contains(BaseFlags::MUT_BORROWED) {
			Err(crate::Error::AlreadyLocked(AlreadyLockedError { _priv: () }))
		} else {
			allocated.add_one_to_borrows();
			Ok(GcRef(*self, std::marker::PhantomData))
		}
	}

	pub fn as_mut(&mut self) -> crate::Result<GcMut<T>> {
		if self.flags().contains(BaseFlags::MUT_BORROWED) || self.upcast().get_borrows() != 0 {
			return Err(crate::Error::AlreadyLocked(AlreadyLockedError { _priv: () }));
		}

		// TODO: soundness bugs with multithreading, as `upcast_mut` could be interrupted
		// and we have two references to it. todo, make baseflags atomic? (same issue with `as_ref`)

		let mut gcmut = GcMut(self.0);
		GcMut::upcast_mut(&mut gcmut).flags().insert(BaseFlags::MUT_BORROWED);

		Ok(gcmut)
	}
}

#[repr(transparent)]
pub struct GcRef<'a, T: 'static>(Gc<T>, std::marker::PhantomData<&'a ()>);

impl<T: Debug + 'static> Debug for GcRef<'_, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_ref(), f)
	}
}

impl<T: 'static> Clone for GcRef<'_, T> {
	fn clone(&self) -> Self {
		self.0.upcast().add_one_to_borrows();
		Self(self.0, std::marker::PhantomData)
	}
}

impl<T: 'static> AsRef<T> for GcRef<'_, T> {
	fn as_ref(&self) -> &T {
		&self
	}
}

impl<T: 'static> Deref for GcRef<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.0.as_ptr() }
	}
}

impl<T: 'static> Drop for GcRef<'_, T> {
	fn drop(&mut self) {
		self.0.upcast().remove_one_from_borrows()
	}
}


#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct GcMut<T: 'static>(NonNull<T>);

impl<T: Debug + 'static> Debug for GcMut<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.as_ref(), f)
	}
}

impl<T: 'static> GcMut<T> {
	pub fn upcast(gc: &Self) -> &Allocated<T> {
		unsafe {
			&*Allocated::upcast(gc.as_ref())
		}
	}

	pub fn upcast_mut(gc: &mut Self) -> &mut Allocated<T> {
		unsafe {
			&mut *Allocated::upcast_mut(gc.as_mut())
		}
	}

	pub fn as_ptr(gc: &Self) -> *const T {
		gc.0.as_ptr() as *const T
	}

	pub fn as_mut_ptr(gc: &mut Self) -> *mut T {
		gc.0.as_ptr()
	}
}

impl<T: 'static> AsRef<T> for GcMut<T> {
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T: 'static> AsMut<T> for GcMut<T> {
	fn as_mut(&mut self) -> &mut T {
		self
	}
}

impl<T: 'static> Deref for GcMut<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { &*Self::as_ptr(self) }
	}
}

impl<T: 'static> DerefMut for GcMut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *Self::as_mut_ptr(self) }
	}
}

impl<T: 'static> Drop for GcMut<T> {
	fn drop(&mut self) {
		Self::upcast_mut(self)
			.flags()
			.remove(BaseFlags::MUT_BORROWED)
	}
}
