use crate::value::ty::{InstanceOf, Singleton};
use crate::value::{AnyValue, Convertible, Value};
use crate::vm::Args;
use crate::Result;
use std::fmt::{self, Debug, Formatter};

pub type Function = for<'a> fn(Args<'a>) -> Result<AnyValue>;

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
	($name:expr, method $func:expr) => {
		RustFn_new!($name, function |obj: $crate::AnyValue, args: $crate::vm::Args<'_>| -> $crate::Result<$crate::AnyValue> {
			$func(obj.try_downcast()?, args)
		})
	};

	($name:expr, function $func:expr) => {
		RustFn_new!($name, justargs |args: crate::vm::Args<'_>| -> $crate::Result<$crate::AnyValue> {
			let (this, args) = args.split_first()?;
			($func)(this, args)
		})
	};
	($name:literal, justargs $func:expr) => {{
		const INNER: &'static $crate::value::ty::rustfn::Inner = &$crate::value::ty::rustfn::Inner {
			name: $name,
			func: $func,
		};

		$crate::value::ty::RustFn::new(INNER)
	}};
	($name:expr, justargs $func:expr) => {{
		const INNER: &'static $crate::value::ty::rustfn::Inner = &$crate::value::ty::rustfn::Inner {
			name: $name.as_str(),
			func: $func,
		};

		$crate::value::ty::RustFn::new(INNER)
	}};
	($_name:expr, $other:tt $_func:expr) => {
		compile_error!(concat!("Unknown rustfn kind '", $other, "'; Please use `method`, `function`, or `justargs`"))
	}
}

impl crate::value::NamedType for RustFn {
	const TYPENAME: &'static str = "RustFn";
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

	pub fn call(self, args: Args<'_>) -> Result<AnyValue> {
		(self.0.func)(args)
	}
}

impl Eq for RustFn {}
impl PartialEq for RustFn {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self.0, rhs.0)
	}
}

impl RustFn {
	pub const NOOP: Self = RustFn_new!("noop", justargs | _ | Ok(Default::default()));
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

impl RustFn {
	pub fn qs_call(self, args: Args<'_>) -> Result<AnyValue> {
		self.call(args)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RustFnClass;

impl Singleton for RustFnClass {
	fn instance() -> crate::AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "RustFn", parent Callable::instance();
				Intern::op_call => method RustFn::qs_call,
			}
		})
	}
}

impl InstanceOf for RustFn {
	type Parent = RustFnClass;
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
