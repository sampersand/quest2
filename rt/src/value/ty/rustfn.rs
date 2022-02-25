use std::fmt::{self, Debug, Formatter};

use crate::value::{AnyValue, Convertible, Value};

type Function = fn(&[u8]) -> crate::Result<AnyValue>;

#[repr(C, align(8))]
pub struct RustFn {
	pub name: &'static str,
	pub func: Function,
	#[doc(hidden)] pub __do_not_touch_me: (),
}

#[macro_export]
macro_rules! RustFn_new {
	($name:expr, $func:expr) => {{
		const RUSTFN: &'static $crate::value::ty::RustFn = &$crate::value::ty::RustFn {
			name: $name,
			func: $func,
			__do_not_touch_me: ()
		};

		RUSTFN
	}};
}

impl RustFn {
	pub const NOOP: &'static Self = RustFn_new!("noop", |_| Ok(Value::NULL.any()));
}

impl Debug for RustFn {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("RustFn")
			.field("name", &self.name)
			.field("func", &(self.func as usize as *const u8))
			.finish()
	}
}

impl From<&'static RustFn> for Value<&'static RustFn> {
	fn from(rustfn: &'static RustFn) -> Self {
		let ptr = rustfn as *const RustFn as usize as u64;

		debug_assert_eq!(ptr & 0b111, 0);

		unsafe { Self::from_bits_unchecked(ptr | 0b100) }
	}
}

unsafe impl Convertible for &'static RustFn {
	type Output = Self;

	fn is_a(value: AnyValue) -> bool {
		value.bits() & 0b111 == 0b100 && value.bits() > 0b011_100
	}

	fn get(value: Value<Self>) -> Self::Output {
		unsafe {
			&*((value.bits() - 0b100) as usize as *const RustFn)
		}
	}
}
