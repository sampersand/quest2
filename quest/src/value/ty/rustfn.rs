use crate::value::ty::{InstanceOf, Singleton};
use crate::value::{Callable, Convertible};
use crate::vm::Args;
use crate::{Result, Value};
use std::fmt::{self, Debug, Formatter};

pub type Function = for<'a> fn(Args<'a>) -> Result<Value>;

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
		RustFn_new!($name, function |obj: $crate::Value, args: $crate::vm::Args<'_>| -> $crate::Result<$crate::Value> {
			$func(obj.try_downcast()?, args)
		})
	};

	($name:expr, function $func:expr) => {
		RustFn_new!($name, justargs |args: crate::vm::Args<'_>| -> $crate::Result<$crate::Value> {
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
			name: $name.as_str_const(),
			func: $func,
		};

		$crate::value::ty::RustFn::new(INNER)
	}};
	($_name:expr, $other:tt $_func:expr) => {
		compile_error!(concat!("Unknown rustfn kind '", $other, "'; Please use `method`, `function`, or `justargs`"))
	}
}

impl crate::value::NamedType for RustFn {
	const TYPENAME: crate::value::Typename = "RustFn";
}

impl RustFn {
	#[doc(hidden)]
	#[must_use]
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
}

impl Callable for RustFn {
	#[inline]
	fn call(self, args: Args<'_>) -> Result<Value> {
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
	pub const NOOP: Self = RustFn_new!("noop", justargs | _ | Ok(Value::default()));
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

		unsafe { Self::from_bits(ptr | 0b1000) }
	}
}

unsafe impl Convertible for RustFn {
	fn is_a(value: Value) -> bool {
		value.bits() & 0b1111 == 0b1000 && value.bits() > 0b1111
	}

	fn get(value: Value<Self>) -> Self {
		unsafe { Self(&*((value.bits() - 0b1000) as usize as *const Inner)) }
	}
}

pub mod funcs {
	use super::*;
	use crate::value::ToValue;

	pub fn call(func: RustFn, args: Args<'_>) -> Result<Value> {
		func.call(args)
	}

	pub fn dbg(func: RustFn, args: Args<'_>) -> Result<Value> {
		use crate::value::ty::text::SimpleBuilder;

		args.assert_no_arguments()?;

		let mut builder = SimpleBuilder::with_capacity(9 + func.name().len());
		builder.push_str("<RustFn:");
		builder.push_str(func.name());
		builder.push('>');

		Ok(builder.finish().to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RustFnClass;

impl Singleton for RustFnClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "RustFn", parent Callable::instance();
				Intern::op_call => method funcs::call,
				Intern::dbg => method funcs::dbg,
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
	use crate::ToValue;

	#[test]
	fn test_is_a() {
		assert!(RustFn::is_a(RustFn::NOOP.to_value()));

		assert!(!RustFn::is_a(Value::TRUE.to_value()));
		assert!(!RustFn::is_a(Value::FALSE.to_value()));
		assert!(!RustFn::is_a(Value::NULL.to_value()));
		assert!(!RustFn::is_a(Value::ONE.to_value()));
		assert!(!RustFn::is_a(Value::ZERO.to_value()));
		assert!(!RustFn::is_a(1.0.to_value()));
		assert!(!RustFn::is_a("hello".to_value()));
	}

	#[test]
	fn test_get() {
		assert_eq!(RustFn::NOOP, RustFn::get(Value::from(RustFn::NOOP)));
	}
}
