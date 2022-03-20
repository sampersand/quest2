use crate::value::gc::Gc;
use crate::vm::{Args, ByteCode};
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Block(Inner);
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	data: Vec<ByteCode>,
	// "source location" ?
}

impl Block {
	#[must_use]
	pub fn new(data: Vec<ByteCode>) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Gc::from_inner(Base::new(Inner { data }, Gc::<Self>::parent()))
	}

	pub fn call(&self, args: Args<'_>) -> Result<AnyValue> {
		let _ = args;
		todo!();
	}
}

impl Default for Gc<Block> {
	fn default() -> Self {
		Block::new(Vec::new())
	}
}

impl AsRef<[ByteCode]> for Block {
	fn as_ref(&self) -> &[ByteCode] {
		&self.0.data().data
	}
}

quest_type_attrs! { for Gc<Block>, parents [Callable, Kernel];
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
