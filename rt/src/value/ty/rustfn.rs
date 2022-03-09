use std::fmt::{self, Debug, Formatter};
use crate::vm::Args;
use crate::value::base::HasDefaultParent;
use crate::Result;
use crate::value::{AnyValue, Convertible, Value};

pub type Function = for<'a> fn(AnyValue, Args<'a>) -> Result<AnyValue>;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct RustFn(&'static Inner);

#[repr(C, align(16))]
#[doc(hidden)]
pub struct Inner {
	pub name: &'static str,
	pub func: Function,
}

#[macro_export]
macro_rules! RustFn_new {
	($name:expr, $func:expr) => {{
		const INNER: &'static $crate::value::ty::rustfn::Inner = &$crate::value::ty::rustfn::Inner {
			name: $name,
			func: $func,
		};

		$crate::value::ty::RustFn::new(INNER)
	}};
}

impl RustFn {
	#[doc(hidden)]
	pub const fn new(inner: &'static Inner) -> Self {
		Self(inner)
	}

	#[must_use]
	pub const fn name(self) -> &'static str {
		self.0.name
	}

	#[must_use]
	pub fn func(self) -> Function {
		self.0.func
	}

	pub fn call(self, obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		(self.0.func)(obj, args)
	}
}

impl Eq for RustFn {}
impl PartialEq for RustFn {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self.0, rhs.0)
	}
}

impl RustFn {
	pub const NOOP: Self = RustFn_new!("noop", |_, _| Ok(Default::default()));
}

impl Debug for RustFn {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "RustFn({:?}:{:p})", self.name(), &(self.func() as usize as *const u8))
	}
}

impl From<RustFn> for Value<RustFn> {
	fn from(rustfn: RustFn) -> Self {
		let ptr = rustfn.0 as *const Inner as usize as u64;

		debug_assert_eq!(ptr & 0b1111, 0);

		unsafe { Self::from_bits_unchecked(ptr | 0b1000) }
	}
}

unsafe impl Convertible for RustFn {
	fn is_a(value: AnyValue) -> bool {
		value.bits() & 0b1111 == 0b1000 && value.bits() > 0b1111
	}

	fn get(value: Value<Self>) -> Self {
		unsafe { Self(&*((value.bits() - 0b1000) as usize as *const Inner)) }
	}
}

impl HasDefaultParent for RustFn {
	fn parent() -> AnyValue {
		Default::default()
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
		assert!(!RustFn::is_a(Default::default()));
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
