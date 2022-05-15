#![allow(
	clippy::wildcard_imports, // used in `funcs` modules
	clippy::unreadable_literal, // there's only a handful and they're not meant to be readable.

	// TODOS:
	clippy::missing_safety_doc,
	clippy::missing_errors_doc,
	clippy::missing_panics_doc,

	// Things that could be issues but aren't
	clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss,

	// Simply my coding style, bite me clippy
	clippy::module_inception,
	clippy::module_name_repetitions,
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
#[must_use]
pub unsafe fn alloc<T>(layout: std::alloc::Layout) -> std::ptr::NonNull<T> {
	debug_assert!(std::alloc::Layout::new::<T>().align() <= layout.align());
	debug_assert!(std::alloc::Layout::new::<T>().size() <= layout.size());

	let ptr = std::alloc::alloc(layout).cast::<T>();

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}

#[allow(clippy::unusual_byte_groupings)]
#[must_use]
pub unsafe fn alloc_zeroed<T>(layout: std::alloc::Layout) -> std::ptr::NonNull<T> {
	debug_assert!(std::alloc::Layout::new::<T>().align() <= layout.align());
	debug_assert!(std::alloc::Layout::new::<T>().size() <= layout.size());

	let ptr = std::alloc::alloc_zeroed(layout).cast::<T>();

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}

#[allow(clippy::unusual_byte_groupings)]
#[must_use]
pub unsafe fn realloc<T>(
	ptr: *mut u8,
	layout: std::alloc::Layout,
	new_size: usize,
) -> std::ptr::NonNull<T> {
	debug_assert!(std::alloc::Layout::new::<T>().align() <= layout.align());
	debug_assert!(std::alloc::Layout::new::<T>().size() <= layout.size());
	debug_assert!(std::alloc::Layout::new::<T>().size() <= new_size);

	let ptr = std::alloc::realloc(ptr, layout, new_size).cast::<T>();

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}
