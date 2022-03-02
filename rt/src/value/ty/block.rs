use crate::value::gc::{Gc, GcRef, Allocated};
use crate::value::base::{Base, Header};
use crate::vm::ByteCode;

#[derive(Debug)]
#[repr(transparent)]
pub struct Block(Base<Inner>);

#[derive(Debug)]
struct Inner {
	data: Vec<ByteCode>
}

impl Gc<Block> {
	pub fn new(data: Vec<ByteCode>) -> Self {
		// unsafe {
			// let mut builder = Self::allocate();
			// builder.data_mut().write(Block { data });
			// Gc::_new(builder.finish())
			let _ = data;
			todo!()
		// }
	}
}

impl Allocated for Block {
	fn header(&self) -> &Header {
		self.0.header()
	}

	fn header_mut(&mut self) -> &mut Header {
		self.0.header_mut()
	}
}

impl Default for Gc<Block> {
	fn default() -> Self {
		Self::new(Vec::new())
	}
}

impl AsRef<[ByteCode]> for GcRef<Block> {
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
