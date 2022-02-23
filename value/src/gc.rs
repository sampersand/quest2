use std::ptr::NonNull;
use crate::{Value, AnyValue, Convertible};
use crate::base::{Flags, Base, Header};

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

	pub unsafe fn from_ref(ptr: &T) -> Self {
		Self::new(ptr.into())
	}

	pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
		&mut *self.0.as_ptr()
	}

	pub unsafe fn as_ref_unchecked(&self) -> &T {
		&*self.0.as_ptr()
	}

	pub fn as_ptr(&self) -> *const T {
		self.0.as_ptr() as *const T
	}

	pub fn header(&self) -> &Header {
		unsafe {
			&*Base::header_for(self.as_ptr())
		}
	}

	pub fn flags(&self) -> &Flags {
		self.header().flags()
	}
}

impl<T: 'static> From<Gc<T>> for Value<T> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b111, 0, "bits mismatch??");

		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl<T: 'static> Convertible for Gc<T> {
	type Inner = T;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		let bits = value.bits();

		if bits & 0b111 != 0 || bits == 0 {
			return false;
		}

		let typeid = unsafe {
			Gc::new(NonNull::new_unchecked(bits as usize as *mut T))
		}.header().typeid();

		typeid == std::any::TypeId::of::<T>()
	}
}
