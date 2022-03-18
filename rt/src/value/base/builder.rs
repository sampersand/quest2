use super::{Attribute, Base};
use crate::AnyValue;
use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::{addr_of_mut, NonNull};

/// The builder used for fine-grained control over making [`Base`]s.
///
/// If you don't need such precise control over [`Base`] creation, consider using [`Base::new`] or
/// [`Base::with_capacity`] instead.
#[must_use]
pub struct Builder<T: 'static>(NonNull<Base<T>>);

impl<T> Builder<T> {
	/// Creates a new `Builder` from a given pointer to the base.
	///
	/// # Safety
	/// For this function to be safe to use, you must ensure the following invariants hold:
	/// - You must ensure that `ptr` is a properly aligned for `Base<T>`.
	/// - You must ensure that `ptr` can be written to.
	/// - While not a strict requirement, `ptr` should be zero-initialized. If not, you will have to
	///   manually initialize _all_ fields yourself before calling [`finish`].
	///
	/// Note that while [`finish`] has some additional safety requirements, they're not technically
	/// required to _create_ a [`Builder`].
	///
	/// # Example
	/// ```
	/// # use qvm_rt::value::base::{Builder, Base};
	/// use std::alloc::{Layout, alloc_zeroed, handle_alloc_error};
	///
	/// /* Allocate the pointer */
	/// let layout = Layout::new::<Base<u8>>();
	///
	/// // SAFETY: we're guaranteed `layout` is not zero-sized,
	/// // as `Base<T>` is nonzero sized.
	/// let ptr = NonNull::new(unsafe { alloc_zeroed(layout) as *mut Base<u8> })
	///     .unwrap_or_else(|| handle_alloc_error(layout));
	///
	/// /* Create the builder */
	/// // SAFETY:
	/// // - We know it's aligned, as we just allocated it with a proper `layout`
	/// // - Likewise, we know we can write to it, as we have just allocated it.
	/// let mut builder = unsafe { Builder::new(ptr) };
	/// builder.write_data(12u8);
	/// 
	/// // SAFETY: Since it's zero-initialized, we only had to initialize the `data`. field.
	/// let base = unsafe { builder.finish() };
	///
	/// assert_eq!(unsafe { *base.as_ptr()).data() }, 12u8);
	/// # std::alloc::free(ptr, layout);
	/// ```
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
			let header = &mut (*this.0.as_ptr()).header;

			if attr_capacity != 0 {
				header.attributes =
					Some(Box::new(super::Attributes::with_capacity(attr_capacity, &header.flags)));
			}
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

	pub fn set_parents<P: super::IntoParent>(&mut self, parents: P) {
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

	pub fn write_data(&mut self, data: T) {
		self.data_mut().write(data);
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> crate::Result<()> {
		self.base_mut().header_mut().set_attr(attr, value)
	}

	#[must_use]
	pub unsafe fn finish(self) -> NonNull<Base<T>> {
		self.0
	}
}
