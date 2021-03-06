use crate::value::base::{Attribute, Builder as BaseBuilder};
use crate::value::{AttributedMut, Gc, HasParents};
use crate::{Result, Value};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Scope(Inner);
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	#[allow(unused)]
	src_loc: crate::vm::SourceLocation,
}

#[must_use]
pub struct Builder(BaseBuilder<Scope>);

impl Builder {
	pub fn with_capacity(cap: usize) -> Self {
		Self(BaseBuilder::with_capacity(cap))
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		self.0.set_attr(attr, value)
	}

	pub fn set_parents<P: crate::value::base::IntoParent>(mut self, parents: P) -> Self {
		self.0.set_parents(parents);
		self
	}

	#[must_use]
	pub fn build(mut self, src_loc: crate::vm::SourceLocation) -> Gc<Scope> {
		self.0.set_data(Inner { src_loc });

		unsafe { self.0.finish() }
	}
}

impl crate::value::gc::Mut<Scope> {
	#[doc(hidden)]
	pub unsafe fn _set_parent_to(&mut self, parent: Value) {
		self.set_parents(parent);
	}
}

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct ScopeClass(());
}

singleton_object! { for ScopeClass, parentof Gc<Scope>, parent Callable;

}
