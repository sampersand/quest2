use crate::value::Gc;
use crate::AnyValue;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct BoundFn(Inner);
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	object: AnyValue,
	function: AnyValue,
}

impl BoundFn {
	pub fn new(object: AnyValue, function: AnyValue) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Gc::from_inner(Base::new(Inner { object, function }, Gc::<Self>::parent()))
	}

	pub fn object(&self) -> AnyValue {
		self.0.data().object
	}

	pub fn function(&self) -> AnyValue {
		self.0.data().function
	}

	pub fn call(&self, args: crate::vm::Args<'_>) -> crate::Result<AnyValue> {
		self.function().call(args.with_self(self.object()))
	}
}

impl Gc<BoundFn> {
	pub fn qs_call(self, args: crate::vm::Args<'_>) -> crate::Result<AnyValue> {
		let (func, obj) = {
			let sref = self.as_ref()?;
			(sref.function(), sref.object())
		};

		func.call(args.with_self(obj))
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
}
