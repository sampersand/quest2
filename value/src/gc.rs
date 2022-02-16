use std::ptr::NonNull;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Gc<T>(NonNull<T>);

impl<T> Gc<T> {
	pub(crate) unsafe fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}

	pub fn as_ref(&self) -> Option<&T> {
		self.as_ptr().map(|x| unsafe{ &*x })
	}

	pub fn as_ptr(&self) -> Option<*const T> {
		Some(self.0.as_ptr())
	}

	// pub fn as_mut(&self)
}
