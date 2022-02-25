use std::fmt::{self, Debug, Formatter};

use crate::{AnyValue, Convertible, Value};

type Function = fn(&[u8]) -> u8;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct RustFn {
	name: &'static str,
	func: Function,
}

impl Debug for RustFn {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("RustFn")
			.field("name", &self.name())
			.field("func", &(self.func() as usize as *const u8))
			.finish()
	}
}

impl RustFn {
	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn func(&self) -> Function {
		self.func
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
		unsafe { &*((value.bits() - 0b100) as usize as *const RustFn) }
	}
}
