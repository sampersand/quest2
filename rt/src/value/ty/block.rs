use crate::{Result, AnyValue};
use crate::value::gc::Gc;
use crate::vm::{Args, ByteCode};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Block(Inner);
}

#[derive(Debug)]
struct Inner {
	data: Vec<ByteCode>,
	// "source location" ?
}

impl Block {
	#[must_use]
	pub fn new(data: Vec<ByteCode>) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		let inner = Base::new_with_parent(data, Gc::<Self>::parent());

		unsafe {
			std::mem::transmute(inner)
		}
	}

	pub fn call(&self, args: Args<'_>) -> Result<AnyValue> {
		let _ = args; todo!();
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
