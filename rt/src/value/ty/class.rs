use crate::value::{Gc, base::Attribute};
use crate::{AnyValue, Result};
use crate::value::ty::Pristine;
use crate::value::base::{Base};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Class(Inner);
}

#[derive(Debug)]
struct Inner {
	name: &'static str
}

pub struct Builder(std::ptr::NonNull<Base<Inner>>);

impl Builder {
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		unsafe { &mut *self.0.as_ptr() }.header_mut().set_attr(attr, value)
	}

	pub fn parent(&mut self, parent: AnyValue) {
		unsafe { &mut *self.0.as_ptr() }.header_mut().set_singular_parent(parent);
	}

	// pub fn function(&mut self, name: &'static str, value: fn(AnyValue, Args<'_>) -> Result<AnyValue>) {
	// 	self.set_attr(name, RustFn_new!(name, value))
	// }

	pub fn finish(self) -> Gc<Class> {
		unsafe {
			std::mem::transmute(self)
		}
	}
}


impl Class {
	pub fn builder(name: &'static str, cap: usize) -> Builder {
		Builder(Base::with_parent_and_capacity(Inner { name }, Pristine::instance(), cap))
	}

	pub const fn name(&self) -> &'static str {
		self.0.data().name
	}
}
