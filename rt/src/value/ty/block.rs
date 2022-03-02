use crate::value::gc::{Gc};
use crate::vm::ByteCode;


quest_type! {
	#[derive(Debug)]
	pub struct Block(Inner);
}

#[derive(Debug)]
struct Inner {
	data: Vec<ByteCode>
}

impl Block {
	pub fn new(data: Vec<ByteCode>) -> Gc<Self> {
		// unsafe {
			// let mut builder = Self::allocate();
			// builder.data_mut().write(Block { data });
			// Gc::new(builder.finish())
			let _ = data;
			todo!()
		// }
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


impl crate::value::base::HasParents for Block {
	unsafe fn init() {
		// todo
	}

	fn parents() -> crate::value::base::Parents {
		Default::default() // todo
	}
}
