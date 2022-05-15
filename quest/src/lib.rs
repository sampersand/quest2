#![allow(
	clippy::wildcard_imports, // used in `funcs` modules
	clippy::deref_addrof, // all the warnings are just from `addr_of!`, which is a valid use.
	clippy::missing_safety_doc, // this is a future TODO
	clippy::module_inception, // just my coding style
)]

extern crate static_assertions as sa;

#[macro_use]
extern crate tracing;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[macro_use]
extern crate qvm_macros;

mod error;

#[macro_use]
pub mod value;
pub mod vm;

#[cfg(test)]
mod integration_tests;
pub mod parser;

pub use error::{Error, Result};
pub use value::{AnyValue, Value};

#[cfg(miri)]
extern "Rust" {
	fn miri_static_root(ptr: *const u8);
}

#[allow(clippy::unusual_byte_groupings)]
pub unsafe fn alloc(layout: std::alloc::Layout) -> std::ptr::NonNull<u8> {
	let ptr = std::alloc::alloc(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}

#[allow(clippy::unusual_byte_groupings)]
pub unsafe fn alloc_zeroed(layout: std::alloc::Layout) -> std::ptr::NonNull<u8> {
	let ptr = std::alloc::alloc_zeroed(layout);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}

#[allow(clippy::unusual_byte_groupings)]
pub unsafe fn realloc(
	ptr: *mut u8,
	layout: std::alloc::Layout,
	new_size: usize,
) -> std::ptr::NonNull<u8> {
	let ptr = std::alloc::realloc(ptr, layout, new_size);

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}
