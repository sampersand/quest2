use crate::value::base::{Attribute, Builder as BaseBuilder};
use crate::value::Gc;
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug)]
	pub struct Scope(Inner);
}

// quest_type! {
// 	pub struct Text(Inner);
// }

// impl super::AttrConversionDefined for Gc<Text> {
// 	const ATTR_NAME: &'static str = "@text";
// }

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

	pub fn build(mut self, src_loc: crate::vm::SourceLocation) -> Gc<Scope> {
		self.0.data_mut().write(Inner { src_loc });

		unsafe { std::mem::transmute(self.0.finish()) }
	}
}

// sa::assert_eq_size!(Scope, ());

impl Scope {
	// pub fn new() -> Gc<Self> {
		
	// }
	// pub const fn new() -> Self {
	// 	Self { _priv: () }
	// }
}

// impl Gc<List> {
// 	pub const fn new() -> Self {

// 	}
// }

impl crate::value::base::HasDefaultParent for Scope {
	fn parent() -> crate::AnyValue {
		Default::default() // todo
	}
}
