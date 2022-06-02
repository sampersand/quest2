use crate::value::{Gc, ToAny};
use crate::Value;
use std::fmt::{self, Debug, Formatter};

quest_type! {
	#[derive(NamedType)]
	pub struct BoundFn(Inner);
}

impl Debug for BoundFn {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("BoundFn")
				.field("object", &self.0.data().object)
				.field("function", &self.0.data().function)
				.finish()
		} else {
			f.debug_tuple("BoundFn")
				.field(&self.0.data().object)
				.field(&self.0.data().function)
				.finish()
		}
	}
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	object: Value,
	function: Value,
}

impl BoundFn {
	#[must_use]
	pub fn new(object: Value, function: Value) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Gc::from_inner(Base::new(Inner { object, function }, Gc::<Self>::parent()))
	}

	pub fn object(&self) -> Value {
		self.0.data().object
	}

	pub fn function(&self) -> Value {
		self.0.data().function
	}

	pub fn call(&self, args: crate::vm::Args<'_>) -> crate::Result<Value> {
		self.function().call(args.with_this(self.object()))
	}
}

impl Gc<BoundFn> {
	pub fn qs_call(self, args: crate::vm::Args<'_>) -> crate::Result<Value> {
		let (func, obj) = {
			let selfref = self.as_ref()?;
			(selfref.function(), selfref.object())
		};

		func.call(args.with_this(obj))
	}

	pub fn dbg(self, args: crate::vm::Args<'_>) -> crate::Result<Value> {
		args.assert_no_arguments()?;

		let selfref = self.as_ref()?;

		let mut builder = crate::value::ty::Text::simple_builder();

		builder.push_str("<BoundFn:");
		builder.push_str(selfref.function().dbg_text()?.as_ref()?.as_str());
		builder.push(':');
		builder.push_str(selfref.object().dbg_text()?.as_ref()?.as_str());
		builder.push('>');

		Ok(builder.finish().to_any())
	}
}

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct BoundFnClass(());
}

singleton_object! {
	for BoundFnClass,
		parentof Gc<BoundFn>,
		parent Callable;

	Intern::op_call => method!(qs_call),
	Intern::dbg => method!(dbg),
}
