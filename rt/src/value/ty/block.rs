use crate::{Result, AnyValue};
use crate::value::gc::Gc;
use crate::vm::{Args, ByteCode};

quest_type! {
	#[derive(Debug)]
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
		// Gc::
		// unsafe {
		// let mut builder = Self::allocate();
		// builder.data_mut().write(Block { data });
		// Gc::new(builder.finish())
		let _ = data;
		todo!()
		// }
	}

	pub fn call(&self, obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let _ = (obj, args); todo!();
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

impl crate::value::base::HasDefaultParent for Block {
	fn parent() -> AnyValue {
		Default::default() // todo
	}
}
