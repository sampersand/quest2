use crate::{ValueBase, Gc};

pub type Integer = i64;
type Inner = std::num::NonZeroU64;


/*
000...0 000 000 = undefined
XXX...X XXX 000 = pointer (nonzero `X`)
XXX...X XXX XX1 = integer
000...0 000 010 = false
000...0 000 100 = true
000...0 000 110 = null
*/
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Value(Inner);

sa::assert_eq_size!(Value, Inner, Integer, *const ());
sa::assert_eq_align!(Value, Inner, Integer, *const ());

sa::assert_eq_size!(Value, Option<Value>);
sa::assert_eq_align!(Value, Option<Value>);


impl Value {
	pub const fn bits(self) -> Inner {
		self.0
	}

	pub const unsafe fn from_bits(bits: Inner) -> Self {
		Self(bits)
	}

	pub const NULL: Self = unsafe { Self::from_bits(Inner::new_unchecked(0b110)) };

	pub const fn new_null() -> Self {
		Self::NULL
	}

	pub const fn is_null(self) -> bool {
		self.bits().get() == Self::NULL.bits().get()
	}

	pub const FALSE: Self = unsafe { Self::from_bits(Inner::new_unchecked(0b_010)) };
	pub const TRUE: Self  = unsafe { Self::from_bits(Inner::new_unchecked(0b_100)) };

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

	pub fn new_base<T>(base: Gc<T>) -> Self {
		let bits = base.as_ptr() as usize as u64;

		assert_eq!(bits & 0b111, 0, "bottom three bits are given for `new_base`?");

		// safety: bits cannot be zero, as `NonNull` is not null
		// safety: this is the definition of a valid `base`.
		unsafe {
			Self::from_bits(Inner::new_unchecked(bits))
		}
	}

	pub fn is_allocated(self) -> bool {
		(self.bits().get() & 0b111) == 0 && self.bits().get() != 0
	}

	pub fn as_base<T>(self) -> Option<*const ValueBase<T>> {
		if !self.is_allocated() {
			return None;
		}

		let ptr = self.bits().get() as *const ValueBase<T>;

		if unsafe { (*ptr).inner_typeid() } == std::any::TypeId::of::<T>() {
			Some(ptr)
		} else {
			None
		}
	}

	pub fn as_allocated<T>(self) -> Option<Gc<T>> {
		if !self.is_allocated() {
			return None;
		}

		// 
		unsafe {
			let ptr = std::ptr::NonNull::new_unchecked(self.bits().get() as *mut T);

			Some(Gc::new(ptr))
		}
	}
}

#[test]
fn test_new_boolean() {
	assert_eq!(Value::new_boolean(true).bits(), Value::TRUE.bits());
	assert_eq!(Value::new_boolean(false).bits(), Value::FALSE.bits());
}
