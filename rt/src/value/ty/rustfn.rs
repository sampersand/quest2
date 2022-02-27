use std::fmt::{self, Debug, Formatter};

use crate::value::{AnyValue, Convertible, Value};

type Function = fn(&[u8]) -> crate::Result<AnyValue>;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RustFn(&'static Inner);

#[repr(C, align(8))]
#[doc(hidden)]
pub struct Inner {
	pub name: &'static str,
	pub func: Function,
}

impl RustFn {
	#[doc(hidden)]
	pub const fn _new(inner: &'static Inner) -> Self {
		Self(inner)
	}

	pub const fn name(&self) -> &'static str {
		self.0.name
	}

	pub fn func(&self) -> Function {
		self.0.func
	}
}

impl Eq for RustFn {}
impl PartialEq for RustFn {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self.0, rhs.0)
	}
}

#[macro_export]
macro_rules! RustFn_new {
	($name:expr, $func:expr) => {{
		const INNER: &'static $crate::value::ty::rustfn::Inner = &$crate::value::ty::rustfn::Inner {
			name: $name,
			func: $func,
		};

		$crate::value::ty::RustFn::_new(INNER)
	}};
}

impl RustFn {
	pub const NOOP: Self = RustFn_new!("noop", |_| Ok(Value::NULL.any()));
}

impl Debug for RustFn {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("RustFn")
			.field("name", &self.name())
			.field("func", &(self.func() as usize as *const u8))
			.finish()
	}
}

impl From<RustFn> for Value<RustFn> {
	fn from(rustfn: RustFn) -> Self {
		let ptr = rustfn.0 as *const Inner as usize as u64;

		debug_assert_eq!(ptr & 0b111, 0);

		unsafe { Self::from_bits_unchecked(ptr | 0b100) }
	}
}

unsafe impl Convertible for RustFn {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() & 0b111 == 0b100 && value.bits() > 0b011_100
	}

	fn get(value: Value<Self>) -> Self::Output {
		unsafe {
			Self(&*((value.bits() - 0b100) as usize as *const Inner))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_a() {
		assert!(RustFn::is_a(Value::from(RustFn::NOOP).any()));

		assert!(!RustFn::is_a(Value::TRUE.any()));
		assert!(!RustFn::is_a(Value::FALSE.any()));
		assert!(!RustFn::is_a(Value::NULL.any()));
		assert!(!RustFn::is_a(Value::ONE.any()));
		assert!(!RustFn::is_a(Value::ZERO.any()));
		assert!(!RustFn::is_a(Value::from(1.0).any()));
		assert!(!RustFn::is_a(Value::from("hello").any()));
	}

	#[test]
	fn test_get() {
		assert_eq!(RustFn::NOOP, RustFn::get(Value::from(RustFn::NOOP)));
	}
}
