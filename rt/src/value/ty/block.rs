use crate::value::gc::Gc;
use crate::vm::{Args, Frame};
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Block(Inner);
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	frame: Frame,
}

impl Block {
	#[must_use]
	pub fn new(frame: Frame) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Gc::from_inner(Base::new(Inner { frame }, Gc::<Self>::parent()))
	}

	pub fn call(&self, args: Args<'_>) -> Result<AnyValue> {
		self.0.data().frame.run(args)
	}
}


quest_type_attrs! { for Gc<Block>, parents [Callable, Kernel];
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
