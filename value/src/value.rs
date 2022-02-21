use crate::{Allocated, Gc};

use std::fmt::{self, Debug, Formatter};

pub type Integer = i64;
type Float = f64;
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
#[derive(Clone, Copy)]
pub struct Value(Inner);

assert_eq_size!(Value, Inner, Integer, *const ());
assert_eq_align!(Value, Inner, Integer, *const ());

assert_eq_size!(Value, Option<Value>);
assert_eq_align!(Value, Option<Value>);

impl Default for Value {
	fn default() -> Self {
		Self::NULL
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if self.is_null() {
			write!(f, "Null")
		} else if let Some(boolean) = self.to_boolean() {
			write!(f, "Boolean({})", boolean)
		} else if let Some(integer) = self.to_integer() {
			write!(f, "Integer({})", integer)
		} else if let Some(float) = self.to_float() {
			write!(f, "Float({})", float)
		} else if let Some(text) = self.to_gc::<crate::text::Text>() {
			Debug::fmt(&text.as_ref().expect("debug asref bad"), f)
		} else if let Some(list) = self.to_gc::<crate::list::List>() {
			Debug::fmt(&list.as_ref().expect("debug asref bad"), f)
		} else if self.is_gc() {
			todo!()
		} else {
			unreachable!("unrecognized tag");
		}
	}
}

impl Value {
	pub const fn bits(self) -> Inner {
		self.0
	}

	pub const unsafe fn from_bits(bits: Inner) -> Self {
		Self(bits)
	}

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


	pub fn new_gc<T>(gc: Gc<T>) -> Self {
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

	pub fn as_base<T>(self) -> Option<*const Allocated<T>> {
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

	pub fn to_gc<T>(self) -> Option<Gc<T>> {
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

#[test]
fn test_new_boolean() {
	assert_eq!(Value::new_boolean(true).bits(), Value::TRUE.bits());
	assert_eq!(Value::new_boolean(false).bits(), Value::FALSE.bits());
}

impl From<Integer> for Value {
	fn from(int: Integer) -> Self {
		Self::new_integer(int)
	}
}

impl From<Float> for Value {
	fn from(float: Float) -> Self {
		Self::new_float(float)
	}
}

impl From<bool> for Value {
	fn from(boolean: bool) -> Self {
		Self::new_boolean(boolean)
	}
}

impl<T: 'static> From<Gc<T>> for Value {
	fn from(gc: Gc<T>) -> Self {
		Self::new_gc(gc)
	}
}
