use std::ptr::NonNull;

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Gc<T: 'static>(NonNull<T>);

impl<T: 'static> Copy for Gc<T> {}
impl<T: 'static> Clone for Gc<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: 'static> Gc<T> {
	pub unsafe fn new(ptr: NonNull<T>) -> Self {
		Self(ptr)
	}

	pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
		&mut *self.0.as_ptr()
	}
}

