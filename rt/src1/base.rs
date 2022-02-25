use crate::{Gc, Attributes};
use std::alloc;
use std::mem::MaybeUninit;
use std::ptr;
use std::cell::UnsafeCell;
use std::any::TypeId;
use std::sync::atomic::{AtomicU32, Ordering};

mod flags;
mod parents;

pub use flags::BaseFlags;
pub(crate) use parents::Parents;

#[repr(C, align(8))]
pub struct Allocated<T: 'static> {
	parents: UnsafeCell<Parents>, // TODO: make me an array
	attributes: Option<Box<Attributes>>,
	typeid: TypeId,
	flags: BaseFlags,
	borrows: AtomicU32,
	data: UnsafeCell<MaybeUninit<T>>
}

assert_eq_size!(Allocated<()>, [u64; 4]);
assert_eq_align!(Allocated<()>, u64);

impl<T: 'static> Allocated<T> {
	pub fn new(data: T) -> *mut Self {
		unsafe {
			let this = Self::allocate();

			(*this).data_mut().write(data);

			this
		}
	}

	/// Safety: you must initialize `T` by calling to `.inner_ptr_mut()` and then writing to it
	/// before you can actually use it or you `drop` it.
	pub unsafe fn allocate() -> *mut Self {
		let layout = alloc::Layout::new::<Self>();

		// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
		// and `attribtues` is valid (as it's zero, which is also `None`).
		let ptr = crate::alloc_zeroed(layout).cast::<Self>();

		// Everything else is default initialized to zero.
		(*ptr).typeid = TypeId::of::<T>();

		ptr
	}

	pub fn data_mut(&mut self) -> &mut MaybeUninit<T> {
		unsafe {
			&mut *self.data.get()
		}
	}

	pub fn data(&self) -> &MaybeUninit<T> {
		unsafe {
			&*self.data.get()
		}
	}

	pub(crate) unsafe fn upcast(ptr: *const T) -> *const Self {
		container_of::container_of!(ptr, Self, data)
	}

	pub(crate) unsafe fn upcast_mut(ptr: *mut T) -> *mut Self {
		container_of::container_of!(ptr, Self, data)
	}

	pub fn inner_typeid(&self) -> TypeId {
		self.typeid
	}

	pub fn flags(&self) -> &BaseFlags {
		&self.flags
	}

	pub fn inner(&self) -> Gc<T> {
		unsafe {
			let data_ptr = self.data().as_ptr();
			let nonnull_data_ptr = ptr::NonNull::new_unchecked(data_ptr as *mut _);
			Gc::new(nonnull_data_ptr)
		}
	}

	pub(crate) fn get_borrows(&self) -> usize {
		self.borrows.load(Ordering::Relaxed) as usize
	}

	pub(crate) fn add_one_to_borrows(&self) {
		let prev = self.borrows.fetch_add(1, Ordering::Relaxed);
		debug_assert_ne!(prev, u32::MAX);
	}

	pub(crate) fn remove_one_from_borrows(&self) {
		let prev = self.borrows.fetch_sub(1, Ordering::Relaxed);
		debug_assert_ne!(prev, 0);
	}
}

impl<T: 'static> Drop for Allocated<T> {
	fn drop(&mut self) {
		unsafe {
			alloc::dealloc(self as *mut _ as _, alloc::Layout::new::<Self>());
		}
	}
}
