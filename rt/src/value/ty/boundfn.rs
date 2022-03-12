use crate::value::Gc;
use crate::AnyValue;

quest_type! {
	#[derive(Debug)]
	pub struct BoundFn(Inner);
}

impl crate::value::NamedType for Gc<BoundFn> {
	const TYPENAME: &'static str = "BoundFn";
}

#[derive(Debug)]
struct Inner {
	object: AnyValue,
	function: AnyValue
}

impl BoundFn {
	pub fn new(object: AnyValue, function: AnyValue) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		let inner = Base::new_with_parent(
			Inner { object, function },
			Gc::<Self>::parent());

		unsafe {
			std::mem::transmute(inner)
		}
	}

	pub fn object(&self) -> AnyValue {
		self.0.data().object
	}

	pub fn function(&self) -> AnyValue {
		self.0.data().function
	}

	pub fn call(&self, args: crate::vm::Args<'_>) -> crate::Result<AnyValue> {
		self.function().call(self.object(), args)
	}
}

impl Gc<BoundFn> {
	pub fn qs_call(self, args: crate::vm::Args<'_>) -> crate::Result<AnyValue> {
		let (func, obj) = {
			let sref = self.as_ref()?;
			(sref.function(), sref.object())
		};

		func.call(obj, args)
	}
}

quest_type_attrs! { for Gc<BoundFn>;
	"()" => meth qs_call,
}
