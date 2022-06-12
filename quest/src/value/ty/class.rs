use crate::value::base::{Attribute, Base, Builder as BaseBuilder};
use crate::value::ty::Pristine;
use crate::value::{AttributedMut, Gc, HasParents};
use crate::{Result, Value};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Class(Inner);
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	name: &'static str,
}

pub struct Builder(BaseBuilder<Inner>);

impl Builder {
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		unsafe { &mut *self.0.as_ptr().as_ptr() }.set_attr(attr, value)
	}

	pub fn parent(&mut self, parent: Value) {
		unsafe { &mut *self.0.as_ptr().as_ptr() }.set_parents(parent);
	}

	// pub fn function(&mut self, name: &'static str, value: fn(Value, Args<'_>) -> Result<Value>) {
	// 	self.set_attr(name, RustFn_new!(name, value))
	// }
	#[must_use]
	pub fn finish(self) -> Gc<Class> {
		unsafe { Gc::from_inner(self.0.finish()) }
	}
}

impl Class {
	#[must_use]
	pub fn builder(name: &'static str, attr_capacity: usize) -> Builder {
		let mut builder = Base::builder();

		builder.set_data(Inner { name });
		builder.set_parents(Pristine::instance());
		builder.allocate_attributes(attr_capacity);

		Builder(builder)
	}

	pub fn name(&self) -> &'static str {
		self.0.data().name
	}
}
