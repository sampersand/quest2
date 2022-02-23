use std::num::NonZeroU64;
use std::marker::PhantomData;

#[repr(transparent)]
pub struct Value<T: ?Sized>(NonZeroU64, PhantomData<T>);

sa::assert_eq_size!(Value<i64>, Value<[u64; 64]>, AnyValue);
sa::assert_eq_align!(Value<i64>, Value<[u64; 64]>, AnyValue);

sa::assert_eq_size!(AnyValue, u64, *const (), Option<AnyValue>);
sa::assert_eq_align!(AnyValue, u64, *const (), Option<AnyValue>);

sa::assert_not_impl_any!(AnyValue: Drop);
sa::assert_not_impl_any!(Value<i64>: Drop);


impl<T> Copy for Value<T> {}
impl<T> Clone for Value<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Value<T> {
	#[inline]
	pub const fn bits(self) -> u64 {
		self.0.get()
	}

	#[inline]
	pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
		Self::from_bits(NonZeroU64::new_unchecked(bits))
	}

	#[inline]
	pub const unsafe fn from_bits(bits: NonZeroU64) -> Self {
		Self(bits, PhantomData)
	}

	#[inline]
	pub const fn any(self) -> AnyValue {
		unsafe { std::mem::transmute(self) }
	}
}

pub struct Any;
pub type AnyValue = Value<Any>;

impl AnyValue {
	#[inline]
	pub fn is_a<T: crate::Convertible>(self) -> bool {
		T::is_a(self)
	}

	#[inline]
	pub fn downcast<T: crate::Convertible<Inner=I>, I>(self) -> Option<Value<I>> {
		T::downcast(self)
	}
}
