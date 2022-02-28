use crate::value::gc::{Gc, GcRef};
use crate::vm::ByteCode;

#[derive(Debug)]
pub struct Block {
	data: Vec<ByteCode>
}

impl Gc<Block> {
	pub fn new(data: Vec<ByteCode>) -> Self {
		unsafe {
			let mut builder = Self::allocate();
			builder.data_mut().write(Block { data });
			builder.finish()
		}
	}
}

impl Default for Gc<Block> {
	fn default() -> Self {
		Self::new(Vec::new())
	}
}

impl AsRef<[ByteCode]> for GcRef<Block> {
	fn as_ref(&self) -> &[ByteCode] {
		&self.data
	}
}


impl crate::value::base::HasParents for Block {
	fn parents() -> crate::value::base::Parents {
		// TODO
		crate::value::base::Parents::NONE
	}
}
