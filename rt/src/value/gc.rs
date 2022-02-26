use crate::value::base::{Base, Flags};
use crate::value::{AnyValue, Convertible, Value};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[repr(transparent)]
pub struct Gc<T: 'static>(NonNull<Base<T>>);

impl<T: 'static> Copy for Gc<T> {}
impl<T: 'static> Clone for Gc<T> {
	fn clone(&self) -> Self {
		*self
	}
}

pub trait Mark {
	fn mark(&self);
}

impl<T> Debug for Gc<T>
where
	GcRef<T>: Debug
{
	fn fmt(self: &Gc<T>, f: &mut Formatter) -> fmt::Result {
		if !f.alternate() {
			if let Some(inner) = self.as_ref() {
				return Debug::fmt(&inner, f);
			}
		}

		write!(f, "Gc({:p}:", self.0)?;

		if let Some(inner) = self.as_ref() {
			Debug::fmt(&inner, f)?;
		} else {
			write!(f, "<locked>")?;
		}

		write!(f, ")")
	}
}

impl<T: 'static> Gc<T> {
	pub unsafe fn _new(ptr: NonNull<Base<T>>) -> Self {
		dbg!(ptr);
		Self(ptr)
	}

	pub unsafe fn as_mut_unchecked(&mut self) -> &mut Base<T> {
		&mut *self.0.as_ptr()
	}

	pub unsafe fn as_ref_unchecked(&self) -> &Base<T> {
		&*self.0.as_ptr()
	}

	pub fn as_ref(self) -> Option<GcRef<T>> {
		// TODO
		Some(GcRef(self))
	}

	pub fn as_mut(self) -> Option<GcMut<T>> {
		// TODO
		Some(GcMut(self))
	}

	pub fn as_ptr(&self) -> *const Base<T> {
		self.0.as_ptr()
	}

	// pub fn header(&self) -> &Header {
	// 	unsafe { &*Base::header_for(self.as_ptr()) }
	// }

	pub fn flags(&self) -> &Flags {
		unsafe {
			&*std::ptr::addr_of!((*self.as_ptr()).flags)
		}
	}
}

impl<T: 'static> From<Gc<T>> for Value<Gc<T>> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b111, 0, "bits mismatch??");

		unsafe { Self::from_bits_unchecked(bits) }
	}
}

unsafe impl<T: 'static> Convertible for Gc<T>
where
	GcRef<T>: Debug
{
	type Output = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		let bits = value.bits();

		if bits & 0b111 != 0 || bits == 0 {
			return false;
		}

		let typeid = unsafe {
			let gc = Gc::_new(NonNull::new_unchecked(bits as usize as *mut Base<()>));
			dbg!(gc.0);
			*std::ptr::addr_of!((*gc.as_ptr()).typeid)
		};

		typeid == std::any::TypeId::of::<T>()
	}

	fn get(value: Value<Self>) -> Self {
		unsafe { Gc::_new(NonNull::new_unchecked(value.bits() as usize as *mut Base<T>)) }
	}
}

#[repr(transparent)]
pub struct GcRef<T: 'static>(Gc<T>);

impl<T: Debug + 'static> Debug for GcRef<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(self.deref(), f)
	}
}

impl<T: 'static> GcRef<T> {
	pub fn as_base_ptr(&self) -> *const Base<T> {
		(self.0).0.as_ptr()
	}

	pub fn flags(&self) -> &Flags {
		unsafe {
			&*std::ptr::addr_of!((*self.as_base_ptr()).flags)
		}
	}
}

impl<T> Deref for GcRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe {
			(*(*self.as_base_ptr()).data.get()).assume_init_ref()
		}
	}
}

impl<T: 'static> Drop for GcRef<T> {
	fn drop(&mut self) {
		// todo
	}
}

#[repr(transparent)]
pub struct GcMut<T: 'static>(Gc<T>);

impl<T: 'static> GcMut<T> {
	pub fn as_mut_base_ptr(&self) -> *mut Base<T> {
		(self.0).0.as_ptr()
	}

	#[inline(always)]
	pub fn r(&self) -> &GcRef<T> {
		unsafe {
			std::mem::transmute(self)
		}
	}
}

impl<T> Deref for GcMut<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.r()
	}
}

impl<T> DerefMut for GcMut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe {
			(*(*self.as_mut_base_ptr()).data.get()).assume_init_mut()
		}
	}
}

impl<T: 'static> Drop for GcMut<T> {
	fn drop(&mut self) {
		// todo
	}
}

