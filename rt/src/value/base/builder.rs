use crate::AnyValue;
use super::{Base, Parents, Attribute};
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of_mut, NonNull};

#[must_use]
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T> Builder<T> {
	// safety: among other things, `ptr` must have been zero initialized (or you have to init it all yourself)
	pub unsafe fn new(ptr: NonNull<Base<T>>) -> Self {
		addr_of_mut!((*ptr.as_ptr()).header.typeid).write(TypeId::of::<T>());

		Self(ptr)
	}

	pub fn inner_ptr(&self) -> NonNull<Base<T>> {
		self.0
	}

	pub fn allocate_with_capacity(attr_capacity: usize) -> Self {
		let this = Self::allocate();

		unsafe {
			(*this.0.as_ptr()).header.attributes.initialize_with_capacity(attr_capacity)
		}

		this
	}

	pub fn allocate() -> Self {
		let layout = std::alloc::Layout::new::<Base<T>>();

		unsafe {
			// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
			// and `attribtues` is valid (as it's zero, which is also `None`).
			Self::new(NonNull::new_unchecked(crate::alloc_zeroed(layout).cast::<Base<T>>()))
		}
	}

	pub unsafe fn _write_parent(&mut self, parent: crate::AnyValue) {
		addr_of_mut!((*self.0.as_ptr()).header.parents).write(Parents { single: parent });
	}

	pub fn write_parents(&mut self, parents: crate::value::Gc<crate::value::ty::List>) {
		self.base_mut().header_mut().set_parents(parents);
	}

	#[inline]
	pub fn base(&self) -> &Base<T> {
		unsafe { self.0.as_ref() }
	}

	#[inline]
	pub fn base_mut(&mut self) -> &mut Base<T> {
		unsafe { self.0.as_mut() }
	}

	#[inline]
	pub fn base_mut_ptr(&mut self) -> *mut Base<T> {
		self.base_mut() as *mut Base<T>
	}

	pub fn flags(&self) -> &super::Flags {
		self.base().header().flags()
	}

	pub fn data(&self) -> &MaybeUninit<T> {
		unsafe { &*self.base().data.get() }
	}

	pub fn data_mut(&mut self) -> &mut MaybeUninit<T> {
		self.base_mut().data.get_mut()
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> crate::Result<()> {
		self.base_mut().header_mut().set_attr(attr, value)
	}

	#[must_use]
	pub unsafe fn finish(self) -> NonNull<Base<T>> {
		self.0
	}
}
