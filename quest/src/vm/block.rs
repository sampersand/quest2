use super::{Frame, SourceLocation};
use crate::value::ty::{List, Text};
use crate::value::{base::Base, Gc, HasDefaultParent};
use crate::vm::Args;
use crate::{AnyValue, Error, Result};
use std::cell::UnsafeCell;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::sync::Arc;

mod builder;
pub use builder::{Builder, Local};

quest_type! {
	#[derive(NamedType)]
	pub struct Block(Arc<BlockInner>);
}

impl Debug for Block {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Block({:p}:{:?})", self, self.0.data().loc)
	}
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
	fn _new(
		code: Vec<u8>,
		loc: SourceLocation,
		constants: Vec<AnyValue>,
		num_of_unnamed_locals: usize,
		named_locals: Vec<Gc<Text>>,
		parent_scope: Option<AnyValue>,
	) -> Gc<Self> {
		let inner = Arc::new(BlockInner {
			block: UnsafeCell::new(MaybeUninit::uninit()),
			code,
			loc,
			constants,
			num_of_unnamed_locals,
			named_locals,
		});

		let gc = Gc::from_inner(if let Some(parent_scope) = parent_scope {
			Base::new(inner.clone(), List::from_slice(&[parent_scope, Gc::<Block>::parent()]))
		} else {
			Base::new(inner.clone(), Gc::<Block>::parent())
		});

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
		Frame::new(self, args)?.run()
	}
}

quest_type_attrs! { for Gc<Block>, parent Object;
	op_call => meth Gc::<Block>::run,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
