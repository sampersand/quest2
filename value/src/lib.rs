extern crate static_assertions as sa;

pub mod base;
mod gc;
pub mod ty;
mod value;

pub use gc::Gc;
pub use value::{AnyValue, Value};

pub unsafe trait Convertible: Into<Value<Self>> {
	type Output: std::fmt::Debug;

	fn is_a(value: AnyValue) -> bool;
	fn downcast(value: AnyValue) -> Option<Value<Self>> {
		if Self::is_a(value) {
			Some(unsafe { std::mem::transmute(value) })
		} else {
			None
		}
	}

	fn get(value: Value<Self>) -> Self::Output;
}

pub unsafe trait Allocated: Sized + 'static {}
// pub unsafe trait Allocated: Sized + 'static
// where
// 	Gc<Self>: Convertible
// {
// 	fn get(value: Value<Self>) -> Gc<Self> {
// 		Gc::new
// 	}
// }

mod private {
	use super::*;

	// FIXME: people can still access immediates.
	pub trait Immediate: Convertible + Copy {
		fn get(value: Value<Self>) -> Self;
	}
}

pub use private::Immediate;

unsafe fn alloc(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc(layout);

	if ptr.is_null() {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

unsafe fn alloc_zeroed(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc_zeroed(layout);

	if ptr.is_null() {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

unsafe fn realloc(ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
	let ptr = std::alloc::realloc(ptr, layout, new_size);

	if ptr.is_null() {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}
