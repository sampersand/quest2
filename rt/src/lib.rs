extern crate static_assertions as sa;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

mod error;

#[macro_use]
pub mod value;

mod init_hooks;
pub mod vm;

pub use error::{Error, Result};
pub use init_hooks::init;
pub use value::{AnyValue, Value};

#[allow(clippy::unusual_byte_groupings)]
unsafe fn alloc(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

#[allow(clippy::unusual_byte_groupings)]
unsafe fn alloc_zeroed(layout: std::alloc::Layout) -> *mut u8 {
	let ptr = std::alloc::alloc_zeroed(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}

#[allow(clippy::unusual_byte_groupings)]
unsafe fn realloc(ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
	let ptr = std::alloc::realloc(ptr, layout, new_size);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	ptr
}
