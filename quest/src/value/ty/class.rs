use crate::value::base::{Attribute, Base, Builder as BaseBuilder};
use crate::value::ty::Pristine;
use crate::value::Gc;
use crate::{AnyValue, Result};

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
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		unsafe { &mut *self.0.as_ptr().as_ptr() }
			.header_mut()
			.set_attr(attr, value)
	}

	pub fn parent(&mut self, parent: AnyValue) {
		unsafe { &mut *self.0.as_ptr().as_ptr() }
			.header()
			.parents()
			.expect("parents shouldnt be locked in the builder")
			.set(parent);
	}

	// pub fn function(&mut self, name: &'static str, value: fn(AnyValue, Args<'_>) -> Result<AnyValue>) {
	// 	self.set_attr(name, RustFn_new!(name, value))
	// }

	pub fn finish(self) -> Gc<Class> {
		unsafe { Gc::from_inner(self.0.finish()) }
	}
}

impl Class {
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
