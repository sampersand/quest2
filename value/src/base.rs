use std::any::TypeId;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU32;

mod attributes;
mod builder;
mod flags;
mod parents;

pub use attributes::Attributes;
pub use builder::Builder;
pub use flags::Flags;
pub use parents::{HasParents, Parents};

#[repr(C, align(8))]
#[derive(Debug)]
pub struct Header {
    parents: UnsafeCell<Parents>, // TODO: make me an array
    attributes: Option<Box<Attributes>>,
    typeid: TypeId,
    flags: Flags,
    borrows: AtomicU32,
}

#[repr(C, align(8))]
#[derive(Debug)]
pub struct Base<T: 'static> {
    header: Header,
    data: UnsafeCell<MaybeUninit<T>>,
}

impl<T: HasParents + 'static> Base<T> {
    pub fn new(data: T) -> crate::Gc<T> {
        unsafe {
            let mut builder = Self::allocate();
            builder.data_mut().write(data);
            builder.finish()
        }
    }

    pub unsafe fn allocate() -> Builder<T> {
        Self::allocate_with_parents(T::parents())
    }
}

impl<T: 'static> Base<T> {
    pub unsafe fn allocate_with_parents(parents: Parents) -> Builder<T> {
        Builder::new(parents)
    }

    // fn data_mut(&mut self) -> &mut MaybeUninit<T> {
    // 	unsafe {
    // 		&mut *self.data.get() // FIXME: can this be simplified?
    // 	}
    // }

    pub fn flags(&self) -> &Flags {
        self.header().flags()
    }

    pub fn typeid(&self) -> TypeId {
        self.header().typeid()
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub unsafe fn upcast(data: *const T) -> *const Self {
        container_of::container_of!(data, Self, data)
    }

    pub unsafe fn header_for(data: *const T) -> *const Header {
        &(*Self::upcast(data)).header as *const Header
    }
}

impl Header {
    pub const fn typeid(&self) -> TypeId {
        self.typeid
    }

    pub const fn flags(&self) -> &Flags {
        &self.flags
    }
}

impl<T> Drop for Base<T> {
    fn drop(&mut self) {
        // TODO: drop data.
    }
}
