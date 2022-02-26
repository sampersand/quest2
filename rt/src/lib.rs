extern crate static_assertions as sa;

mod error;

#[macro_use]
pub mod value;

pub mod vm;

pub use error::{Error, Result};
pub use value::{AnyValue, Value};

unsafe fn alloc(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

unsafe fn alloc_zeroed(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc_zeroed(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

unsafe fn realloc(ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
	let ptr = std::alloc::realloc(ptr, layout, new_size);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}
