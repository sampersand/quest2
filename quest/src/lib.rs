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

extern crate self as quest; // for proc macros

#[macro_use]
extern crate tracing;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[macro_use]
extern crate qvm_macros;

#[macro_use]
pub mod value;
pub mod error;
pub mod parse;
pub mod vm;

pub use error::{Error, ErrorKind, Result};
pub use value::{ToValue, Value};

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
	miri_static_root(ptr.cast()); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}

// SAFETY: layout must be nonzero
#[allow(clippy::unusual_byte_groupings)]
#[must_use]
pub unsafe fn alloc_zeroed<T>(layout: std::alloc::Layout) -> std::ptr::NonNull<T> {
	debug_assert!(std::alloc::Layout::new::<T>().align() <= layout.align());
	debug_assert!(std::alloc::Layout::new::<T>().size() <= layout.size());

	// This should not be used by anyone. It's just me seeing how fast i can _theroetically_
	// get quest if i have everything preallocated. (the size is what's required for `fib(30)`)
	#[cfg(feature = "unsafe-arena-alloc-hack")]
	{
		static mut PTR: *mut u8 = std::ptr::null_mut();

		if PTR.is_null() {
			PTR = std::alloc::alloc_zeroed(
				std::alloc::Layout::from_size_align(1163177848 * 2, 16).unwrap(),
			);
		}

		let result = PTR;
		PTR = PTR.add(layout.size());
		if (PTR as usize) % 16 != 0 {
			PTR = PTR.add(8);
		}
		debug_assert_eq!((PTR as usize) % 16, 0);
		return std::ptr::NonNull::new_unchecked(result.cast());
	}

	let ptr = std::alloc::alloc_zeroed(layout).cast::<T>();

	if ptr.is_null() || (ptr as u64 <= 0b111_111) {
		std::alloc::handle_alloc_error(layout);
	}

	#[cfg(miri)]
	miri_static_root(ptr.cast()); // TODO: garbage collection

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
	miri_static_root(ptr.cast()); // TODO: garbage collection

	std::ptr::NonNull::new_unchecked(ptr)
}
