use std::mem::MaybeUninit;
use std::any::TypeId;
use std::sync::atomic::{AtomicU32, Ordering};
use std::cell::UnsafeCell;
use std::alloc;
use crate::Gc;

mod flags;
mod parents;
mod attributes;

pub use flags::Flags;
pub use attributes::Attributes;
use parents::Parents;

#[repr(C, align(8))]
pub struct Base<T: 'static> {
	parents: UnsafeCell<Parents>, // TODO: make me an array
	attributes: Option<Box<Attributes>>,
	typeid: TypeId,
	flags: Flags,
	borrows: AtomicU32,
	data: UnsafeCell<MaybeUninit<T>>
}

impl<T: 'static> Base<T> {
	pub unsafe fn allocate() -> BaseBuilder<T> {
		let layout = alloc::Layout::new::<Self>();

		// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
		// and `attribtues` is valid (as it's zero, which is also `None`).
		let ptr = alloc::alloc_zeroed(layout).cast::<Self>();

		// Everything else is default initialized to zero.
		(*ptr).typeid = TypeId::of::<T>();

		BaseBuilder(ptr)
	}

	fn data_mut(&mut self) -> &mut MaybeUninit<T> {
		unsafe {
			&mut *self.data.get() // FIXME: can this be simplified?
		}
	}

	pub fn flags(&self) -> &Flags {
		&self.flags
	}
}

pub trait Allocated : 'static + Sized {
	unsafe fn allocate() -> BaseBuilder<Self> {
		Base::allocate()
	}
	// fn new(self) -> Gc<Self>
}

#[must_use]
pub struct BaseBuilder<T: 'static>(*mut Base<T>);

impl<T> BaseBuilder<T> {
	pub fn base(&mut self) -> &mut Base<T> {
		unsafe {
			&mut *self.0
		}
	}
	pub fn data(&mut self) -> &mut MaybeUninit<T> {
		unsafe {
			&mut *(*self.0).data.get()
		}
	}

	pub unsafe fn finish(self) -> Gc<T> {
		// TODO: use `assume_init`
		Gc::new(std::ptr::NonNull::new_unchecked(self.0 as *mut T))
	}
}

// impl<T: 'static> Into<Value<Self>> for Base<T> {
// 	fn into()
// }

// pub unsafe trait Convertible : Into<Value<Self>> {
// 	fn is_a(value: AnyValue) -> bool;
// 	fn downcast(value: AnyValue) -> Option<Value<Self>> {
// 		if Self::is_a(value) {
// 			Some(unsafe { std::mem::transmute(value) })
// 		} else {
// 			None
// 		}
// 	}
// }
