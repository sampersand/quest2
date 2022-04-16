use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use crate::{AnyValue, Result};
use crate::value::{Gc, ty::Text, HasDefaultParent};
use crate::vm::Args;
use std::sync::Arc;
use super::{Frame, SourceLocation};


quest_type! {
	#[derive(NamedType, Debug)]
	pub struct Block(Arc<BlockInner>);
}

#[derive(Debug)]
pub struct BlockInner {
	pub(super) block: UnsafeCell<MaybeUninit<Gc<Block>>>,
	pub(super) code: Vec<u8>,
	pub(super) loc: SourceLocation,
	pub(super) constants: Vec<AnyValue>,
	pub(super) num_of_unnamed_locals: usize,
	pub(super) named_locals: Vec<Gc<Text>>,
}

impl Block {
	pub fn _new(code: Vec<u8>, loc: SourceLocation, constants: Vec<AnyValue>,
		num_of_unnamed_locals: usize, named_locals: Vec<Gc<Text>>
	) -> Gc<Self> {
		let inner = Arc::new(BlockInner {
			block: UnsafeCell::new(MaybeUninit::uninit()),
			code, loc, constants, num_of_unnamed_locals, named_locals
		});
		let gc = Gc::from_inner(crate::value::base::Base::new(inner.clone(), Gc::<Block>::parent()));
		unsafe {
			inner.block.get().write(MaybeUninit::new(gc));
		}
		gc
	}

	pub(crate) fn inner(&self) -> Arc<BlockInner> {
		self.0.data().clone()
	}
}

impl Gc<Block> {
	pub fn run(self, args: Args) -> Result<AnyValue> {
		let frame = Frame::new(self, args)?;

		match frame.run() {
			Err(crate::Error::Return { value, from_frame })
				if from_frame.is_identical(frame.into()) => Ok(value),
			other => other
		}
	}
}

quest_type_attrs! { for Gc<Block>;
	op_call => meth Gc::<Block>::run,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
