use crate::Result;
use crate::value::base::{Base, Builder, Flags, HasParents};
use crate::value::{AnyValue, Convertible, Value};
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};

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

const MUT_BORROW: u32 = u32::MAX;

impl<T> Debug for Gc<T>
where
	GcRef<T>: Debug,
{
	fn fmt(self: &Gc<T>, f: &mut Formatter) -> fmt::Result {
		if !f.alternate() {
			if let Ok(inner) = self.as_ref() {
				return Debug::fmt(&inner, f);
			}
		}

		write!(f, "Gc({:p}:", self.0)?;

		if let Ok(inner) = self.as_ref() {
			Debug::fmt(&inner, f)?;
		} else {
			write!(f, "<locked>")?;
		}

		write!(f, ")")
	}
}

impl<T: HasParents + 'static> Gc<T> {
	pub unsafe fn allocate() -> Builder<T> {
		Base::allocate()
	}
}

impl<T: 'static> Gc<T> {
	pub unsafe fn _new(ptr: NonNull<Base<T>>) -> Self {
		Self(ptr)
	}

	pub unsafe fn _new_unchecked(ptr: *mut Base<T>) -> Self {
		Self::_new(NonNull::new_unchecked(ptr))
	}

	pub fn as_ref(self) -> crate::Result<GcRef<T>> {
		fn updatefn(x: u32) -> Option<u32> {
			if x == MUT_BORROW {
				None
			} else {
				Some(x + 1)
			}
		}

		if self
			.borrows()
			.fetch_update(Ordering::Acquire, Ordering::Relaxed, updatefn)
			.is_ok()
		{
			Ok(GcRef(self))
		} else {
			Err(crate::Error::AlreadyLocked(Value::from(self).any()))
		}
	}

	pub fn as_mut(self) -> crate::Result<GcMut<T>> {
		if self.flags().contains(Flags::FROZEN) {
			return Err(crate::Error::ValueFrozen(Value::from(self).any()))
		}

		if self
			.borrows()
			.compare_exchange(0, MUT_BORROW, Ordering::Acquire, Ordering::Relaxed)
			.is_ok()
		{
			Ok(GcMut(self))
		} else {
			Err(crate::Error::AlreadyLocked(Value::from(self).any()))
		}
	}

	pub fn as_ptr(&self) -> *const Base<T> {
		self.0.as_ptr()
	}

	fn flags(&self) -> &Flags {
		unsafe { &*std::ptr::addr_of!((*self.as_ptr()).header.flags) }
	}

	fn borrows(&self) -> &AtomicU32 {
		unsafe { &*std::ptr::addr_of!((*self.as_ptr()).header.borrows) }
	}
}

impl<T: 'static> From<Gc<T>> for Value<Gc<T>> {
	#[inline]
	fn from(text: Gc<T>) -> Self {
		let bits = text.as_ptr() as usize as u64;
		debug_assert_eq!(bits & 0b111, 0, "misaligned?!");

		unsafe { Self::from_bits_unchecked(bits) }
	}
}

unsafe impl<T: 'static> Convertible for Gc<T>
where
	GcRef<T>: Debug,
{
	type Output = Self;

	#[inline]
	fn is_a(value: AnyValue) -> bool {
		let bits = value.bits();

		if bits & 0b111 != 0 || bits == 0 {
			return false;
		}

		let typeid = unsafe {
			let gc = Gc::_new_unchecked(bits as usize as *mut Base<()>);
			*std::ptr::addr_of!((*gc.as_ptr()).header.typeid)
		};

		typeid == std::any::TypeId::of::<T>()
	}

	fn get(value: Value<Self>) -> Self {
		unsafe { Gc::_new_unchecked(value.bits() as usize as *mut Base<T>) }
	}
}

impl<T: 'static> GcRef<T> {
	pub fn header(&self) -> &crate::value::base::Header {
		&self.base().header
	}

	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.header().attributes.get_attr(attr)
	}
}

impl<T: 'static> GcMut<T> {
	pub fn header_mut(&mut self) -> &mut crate::value::base::Header {
		&mut self.base_mut().header
	}

	pub fn parents(&mut self) -> Gc<crate::value::ty::List> {
		self.header_mut().attributes.parents.as_list()
	}

	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		self.header_mut().attributes.set_attr(attr, value)
	}

	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		self.header_mut().attributes.del_attr(attr)
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
	pub fn as_gc(&self) -> Gc<T> {
		self.0
	}

	fn base(&self) -> &Base<T> {
		unsafe { &*self.as_base_ptr() }
	}

	pub fn as_base_ptr(&self) -> *const Base<T> {
		(self.0).0.as_ptr()
	}

	pub fn flags(&self) -> &Flags {
		unsafe { &*std::ptr::addr_of!((*self.as_base_ptr()).header.flags) }
	}

	pub fn freeze(&self) {
		self.flags().insert(Flags::FROZEN);
	}

	pub fn is_frozen(&self) -> bool {
		self.flags().contains(Flags::FROZEN)
	}
}

impl<T> Deref for GcRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		unsafe { (*(*self.as_base_ptr()).data.get()).assume_init_ref() }
	}
}

impl<T: 'static> Drop for GcRef<T> {
	fn drop(&mut self) {
		let prev = self.0.borrows().fetch_sub(1, Ordering::SeqCst);

		debug_assert_ne!(prev, MUT_BORROW);
		debug_assert_ne!(prev, 0);
	}
}

#[repr(transparent)]
pub struct GcMut<T: 'static>(Gc<T>);

impl<T: 'static> GcMut<T> {
	pub fn base_mut(&self) -> &mut Base<T> {
		unsafe { &mut *self.as_mut_base_ptr() }
	}

	pub fn as_mut_base_ptr(&self) -> *mut Base<T> {
		(self.0).0.as_ptr()
	}

	#[inline(always)]
	pub fn r(&self) -> &GcRef<T> {
		unsafe { std::mem::transmute(self) }
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
		unsafe { (*(*self.as_mut_base_ptr()).data.get()).assume_init_mut() }
	}
}

impl<T: 'static> Drop for GcMut<T> {
	fn drop(&mut self) {
		let prev = self.0.borrows().swap(0, Ordering::Release);
		debug_assert_eq!(prev, MUT_BORROW);
	}
}

#[cfg(test)]
mod tests {
	use crate::Error;
	use super::*;

	#[test]
	fn respects_refcell_rules() {
		let text = Gc::from_str("g'day mate");

		let mut1 = text.as_mut().unwrap();
		assert_matches!(text.as_ref(), Err(Error::AlreadyLocked(_)));
		drop(mut1);

		let ref1 = text.as_ref().unwrap();
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		let ref2 = text.as_ref().unwrap();
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		drop(ref1);
		assert_matches!(text.as_mut(), Err(Error::AlreadyLocked(_)));

		drop(ref2);
		assert_matches!(text.as_mut(), Ok(_));
	}

	#[test]
	fn respects_frozen() {
		let text = Gc::from_str("Hello, world");

		text.as_mut().unwrap().push('!');
		assert_eq!(text.as_ref().unwrap(), *"Hello, world!");
		assert!(!text.as_ref().unwrap().is_frozen());

		text.as_ref().unwrap().freeze();
		assert_matches!(text.as_mut(), Err(Error::ValueFrozen(_)));
		assert!(text.as_ref().unwrap().is_frozen());
	}
}
