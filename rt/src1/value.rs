use crate::{Allocated, Gc, kinds, Convertible, Immediate, Allocation};
use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};

pub type Integer = i64;
type Float = f64;
use std::num::NonZeroU64;

type Inner = std::num::NonZeroU64;

/*
000...0 000 000 = undefined
XXX...X XXX 000 = pointer (nonzero `X`)
XXX...X XXX XX1 = i63
XXX...X XXX X10 = f62
000...0 000 100 = false
000...0 001 100 = null
000...0 010 100 = true
*/
#[repr(transparent)]
pub struct Value<T: ?Sized>(Inner, PhantomData<T>);

pub type AnyValue = Value<()>;

impl<T> Copy for Value<T>{}
impl<T> Clone for Value<T> {
	fn clone(&self) -> Self {
		*self
	}
}

assert_eq_size!(AnyValue, Inner, Integer, *const ());
assert_eq_align!(AnyValue, Inner, Integer, *const ());

assert_eq_size!(AnyValue, Option<AnyValue>);
assert_eq_align!(AnyValue, Option<AnyValue>);

impl Default for AnyValue {
	fn default() -> Self {
		Value::<kinds::Null>::default().any()
	}
}
impl Default for Value<kinds::Null> {
	fn default() -> Self {
		kinds::Null.into()
	}
}

impl Debug for AnyValue {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.is_a::<crate::kinds::Null>() {
			write!(f, "Null")
		} else if let Some(boolean) = self.downcast::<crate::kinds::Boolean>() {
			write!(f, "Boolean({})", boolean)
		} else if let Some(integer) = self.downcast::<Integer>() {
			write!(f, "Integer({})", integer)
		} else if let Some(float) = self.downcast::<Float>() {
			write!(f, "Float({})", float)
		} else if let Some(text) = self.downcast::<crate::kinds::Text>() {
			Debug::fmt(&text.as_ref().expect("debug asref bad"), f)
		} else if let Some(list) = self.downcast::<crate::kinds::List>() {
			Debug::fmt(&list.as_ref().expect("debug asref bad"), f)
		} else if self.is_gc() {
			todo!()
		} else {
			unreachable!("unrecognized tag");
		}
	}
}

impl AnyValue {
	pub fn is_a<T: Convertible>(self) -> bool {
		T::is_a(self)
	}

	pub fn downcast<T: Convertible>(self) -> Option<Value<T>> {
		T::downcast(self)
	}

	pub fn downcast_imm<T: Convertible + Immediate>(self) -> Option<T> {
		self.downcast().map(T::get)
	}

	pub fn downcast_ref<T: Convertible + Immediate>(self) -> Option<T> {
		self.downcast().map(T::get)
	}
// pub unsafe trait Convertible : Into<Value<Self>> {
// 	fn is_a(value: AnyValue) -> bool;

// 	fn downcast(value: AnyValue) -> Option<Value<Self>> {
// 		if Self::is_a(value) {
// 			Some(unsafe { std::mem::transmute(value) })
// 		} else {
// 			None
// 		}
// 	}
// }

// pub trait Immediate : Convertible + Copy {
// 	fn get(value: Value<Self>) -> Self;
// }

// pub trait Allocation : Convertible {
// 	fn get(value: &Value<Self>) -> &Self;
// }

}

impl<T> Value<T> {
	pub const fn bits(self) -> u64 {
		self.bits_raw().get()
	}

	pub const fn bits_raw(self) -> NonZeroU64 {
		self.0
	}

	pub const unsafe fn from_bits(bits: NonZeroU64) -> Self {
		Self(bits, PhantomData)
	}

	pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
		Self::from_bits(NonZeroU64::new_unchecked(bits))
	}

	pub fn any(self) -> AnyValue {
		unsafe {
			std::mem::transmute(self)
		}
	}
}

/*
impl<T> Value<T> {
	pub const NULL: Self = unsafe { Self::from_bits(Inner::new_unchecked(0b001_100)) };

	pub const fn new_null() -> Self {
		Self::NULL
	}

	pub const fn is_null(self) -> bool {
		self.bits().get() == Self::NULL.bits().get()
	}

	pub const FALSE: Self = unsafe { Self::from_bits(Inner::new_unchecked(0b000_100)) };
	pub const TRUE: Self  = unsafe { Self::from_bits(Inner::new_unchecked(0b010_100)) };

	pub const fn new_boolean(bool: bool) -> Self {
		if bool {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}

	pub const fn is_boolean(self) -> bool {
		self.bits().get() == Self::FALSE.bits().get()
			|| self.bits().get() == Self::TRUE.bits().get()
	}

	pub const fn to_boolean(self) -> Option<bool> {
		if self.bits().get() == Self::TRUE.bits().get() {
			Some(true)
		} else if self.bits().get() == Self::FALSE.bits().get() {
			Some(false)
		} else {
			None
		}
	}

	pub const ZERO: Self = Self::new_integer(0);
	pub const ONE: Self = Self::new_integer(1);

	pub const fn new_integer(int: i64) -> Self {
		// TODO: debug check if `int` is too large and we're truncating.

		// SAFETY: we always `|` with `1`, so it's never zero.
		unsafe {
			let bits = Inner::new_unchecked(((int << 1) | 1) as _);
			Self::from_bits(bits)
		}
	}

	pub const fn is_integer(self) -> bool {
		(self.bits().get() & 1) == 1
	}

	pub const fn to_integer(self) -> Option<Integer> {
		if self.is_integer() {
			Some((self.bits().get() as Integer) >> 1)
		} else {
			None
		}
	}

	pub fn new_float(float: Float) -> Self {
		// SAFETY: we always `|` with `2`, so it's never zero.
		unsafe {
			let bits = Inner::new_unchecked((float.to_bits() & !3) | 2);
			Self::from_bits(bits)
		}
	}

	pub const fn is_float(self) -> bool {
		(self.bits().get() & 3) == 2
	}

	pub fn to_float(self) -> Option<Float> {
		if self.is_float() {
			Some(Float::from_bits(self.bits().get() & !2))
		} else {
			None
		}
	}

	pub fn new_gc(gc: Gc<T>) -> Self {
		let bits = unsafe { gc.as_ptr() as usize as u64 };

		debug_assert_eq!(bits & 0b111, 0, "bottom three bits are given for `new_base`?");

		// safety: bits cannot be zero, as `NonNull` is not null
		// safety: this is the definition of a valid `gc`.
		unsafe {
			Self::from_bits(Inner::new_unchecked(bits))
		}
	}

	pub fn is_gc(self) -> bool {
		(self.bits().get() & 0b111) == 0 && self.bits().get() != 0
	}

	pub fn as_base(self) -> Option<*const Allocated<T>> /* where T: allocated */ {
		if !self.is_gc() {
			return None;
		}

		let ptr = self.bits().get() as *const Allocated<T>;

		if unsafe { (*ptr).inner_typeid() } == std::any::TypeId::of::<T>() {
			Some(ptr)
		} else {
			None
		}
	}

	pub fn to_gc(self) -> Option<Gc<T>> {
		if !self.is_gc() {
			return None;
		}

		let ptr = unsafe {
			let raw = self.bits().get() as *mut T;
			Gc::new(std::ptr::NonNull::new_unchecked(raw))
		};

		if ptr.upcast().inner_typeid() == std::any::TypeId::of::<T>() {
			Some(ptr)
		} else {
			None
		}
	}
}
*/

#[test]
fn test_new_boolean() {
	assert_eq!(Value::new_boolean(true).bits(), Value::TRUE.bits());
	assert_eq!(Value::new_boolean(false).bits(), Value::FALSE.bits());
}

impl<T: 'static> From<Gc<T>> for Value<T> {
	fn from(gc: Gc<T>) -> Self {
		Self::new_gc(gc)
	}
}
