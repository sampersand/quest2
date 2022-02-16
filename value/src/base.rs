use std::collections::HashMap;
use crate::{Value, Gc};
use std::alloc;
use std::mem::{size_of, align_of};
use std::marker::PhantomData;
use std::any::TypeId;
use std::sync::atomic::AtomicU64;

mod flags;
pub use flags::Flags;

#[repr(C, align(8))]
pub struct ValueBase<T: 'static> {
	parent: Option<Value>,
	attributes: Option<Box<HashMap<String, Value>>>,
	typeid: TypeId,
	flags: BaseFlags, // these can be used by anyone.
	_data: PhantomData<T>
}

const fn layout_for<T: 'static>() -> alloc::Layout {
	let mut size = size_of::<ValueBase<T>>() + size_of::<T>();

	let align =
		if align_of::<ValueBase<T>>() >= align_of::<T>() {
			align_of::<ValueBase<T>>()
		} else {
			size += align_of::<T>() - align_of::<ValueBase<T>>(); // we need padding
			align_of::<T>()
		};

	match alloc::Layout::from_size_align(size, align) {
		Ok(value) => value,
		Err(_err) => panic!("cannot create layout")
	}
}

impl<T: 'static> ValueBase<T> {
	/// safety: you must initialize `T` by calling to `.inner_ptr_mut()` and then writing to it
	/// before you can actually use it or you `drop` it.
	pub unsafe fn allocate() -> *mut Self {
		let layout = layout_for::<T>();

		// Since we `alloc_zeroed`, `parent` is valid (as it's zero, which is `None`),
		// and `attribtues` is valid (as it's zero, which is also `None`).
		let ptr = alloc::alloc_zeroed(layout) as *mut Self;

		(*ptr).typeid = TypeId::of::<T>();

		ptr
	}

	pub unsafe fn upcast(ptr: &T) -> &Self {
		&*((ptr as *const _ as *const u8).offset(size_of::<Self>() as _) as *const Self)
	}

	pub fn inner_typeid(&self) -> TypeId {
		self.typeid
	}

	pub fn flags(&self) -> BaseFlags {
		self.flags
	}

	pub fn flags_mut(&mut self) -> &mut BaseFlags {
		&mut self.flags
	}

	pub fn inner(&self) -> Gc<T> {
		Gc::new(std::ptr::NonNull::new(self.inner_ptr() as *mut _).unwrap())
	}

	pub fn inner_ptr(&self) -> *const T {
		sa::assert_eq_size!(ValueBase<()>, [u64; 4]);
		sa::assert_eq_align!(ValueBase<()>, [u64; 4]);

		let mut end_ptr =
			unsafe {
				// safety: we're within an allocated object.
				(self as *const _ as *const u8).offset(size_of::<Self>() as _)
			};

		if align_of::<ValueBase<T>>() < align_of::<T>() {
			end_ptr = 
				unsafe {
					end_ptr.offset((align_of::<T>() - align_of::<ValueBase<T>>()) as _)
				};
		}

		end_ptr as *const T
	}

	pub fn inner_ptr_mut(&mut self) -> *mut T {
		self.inner_ptr() as *mut T
	}
}

impl<T: 'static> AsRef<T> for ValueBase<T> {
	fn as_ref(&self) -> &T {
		unsafe {
			&*self.inner_ptr()
		}
	}
}

impl<T: 'static> AsMut<T> for ValueBase<T> {
	fn as_mut(&mut self) -> &mut T {
		unsafe {
			&mut *self.inner_ptr_mut()
		}
	}
}

impl<T: 'static> Drop for ValueBase<T> {
	fn drop(&mut self) {
		unsafe {
			self.inner_ptr_mut().drop_in_place();
			alloc::dealloc(self as *mut _ as _, layout_for::<T>());
		}
	}
}
