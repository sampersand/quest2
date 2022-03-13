use crate::value::base::{Attribute, Builder as BaseBuilder};
use crate::value::Gc;
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Scope(Inner);
}

#[derive(Debug)]
struct Inner {
	#[allow(unused)]
	src_loc: crate::vm::SourceLocation
}

pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	pub fn with_capacity(cap: usize) -> Self {
		Self(BaseBuilder::allocate_with_capacity(cap))
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		self.0.set_attr(attr, value)
	}

	pub fn parent(mut self, parent: AnyValue) -> Self {
		unsafe { self.0._write_parent(parent); }
		self
	}

	pub fn parents(mut self, parents: crate::value::Gc<crate::value::ty::List>) -> Self {
		self.0.write_parents(parents);
		self
	}

	pub fn build(mut self, src_loc: crate::vm::SourceLocation) -> Gc<Scope> {
		self.0.data_mut().write(Inner { src_loc });

		unsafe { std::mem::transmute(self.0.finish()) }
	}
}

impl crate::value::gc::Mut<Scope> {
	#[doc(hidden)]
	pub unsafe fn _set_parent_to(&mut self, parent: AnyValue) {
		use crate::value::gc::Allocated;

		self.header_mut().set_singular_parent(parent);
	}
}

quest_type_attrs! { for Gc<Scope>, parent Object;

}
