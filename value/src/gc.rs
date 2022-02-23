use std::ptr::NonNull;
use crate::{Value, AnyValue, Convertible};
use crate::base::{Flags, Base, Header};
use std::fmt::{self, Debug, Formatter};

#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct Gc<T: 'static>(NonNull<T>);

impl<T: 'static> Copy for Gc<T> {}
impl<T: 'static> Clone for Gc<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Debug> Debug for Gc<T> {
	fn fmt(self: &Gc<T>, f: &mut Formatter) -> fmt::Result {
		if !f.alternate() {
			if let Some(inner) = self.as_ref() {
				return Debug::fmt(&*inner, f);
			}
		}

		write!(f, "Gc({:p}:", self.0)?;

		if let Some(inner) = self.as_ref() {
			Debug::fmt(&*inner, f)?;
		} else {
			write!(f, "<locked>")?;
		}

		write!(f, ")")
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

	pub fn as_ref(&self) -> Option<impl std::ops::Deref<Target=T> + '_> {
		// TODO
		Some(unsafe {
			self.as_ref_unchecked()
		})
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

impl<T: 'static> From<Gc<T>> for Value<Gc<T>> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b111, 0, "bits mismatch??");

		unsafe {
			Self::from_bits_unchecked(bits)
		}
	}
}

unsafe impl<T: Debug + 'static> Convertible for Gc<T> {
	type Output = Self;

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

	fn get(value: Value<Self>) -> Self {
		unsafe {
			Gc::new(NonNull::new_unchecked(value.bits() as usize as *mut T))
		}
	}
}
