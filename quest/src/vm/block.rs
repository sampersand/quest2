use super::{Frame, SourceLocation};
use crate::value::ty::{List, Text};
use crate::value::{base::Base, Gc, HasDefaultParent};
use crate::vm::Args;
use crate::{AnyValue, Error, Result};
use std::fmt::{self, Debug, Formatter};
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
			code,
			loc,
			constants,
			num_of_unnamed_locals,
			named_locals,
		});

		Gc::from_inner(if let Some(parent_scope) = parent_scope {
			Base::new(inner, List::from_slice(&[parent_scope, Gc::<Self>::parent()]))
		} else {
			Base::new(inner, Gc::<Self>::parent())
		})
	}

	pub(crate) fn inner(&self) -> Arc<BlockInner> {
		self.0.data().clone()
	}
}

impl Gc<Block> {
	pub fn run(self, args: Args<'_>) -> Result<AnyValue> {
		Frame::new(self, args)?.run()
	}

	pub fn deep_clone(&self) -> Result<Self> {
		// TODO: optimize me, eg maybe have shared attributes pointer or something
		let selfref = self.as_ref()?;
		let inner = selfref.inner().clone();
		let parents = selfref.parents().unwrap();
		let cloned = Self::from_inner(Base::new(inner, parents));

		let mut clonedmut = cloned.as_mut().unwrap();
		for (attr, value) in selfref.attributes()?.iter() {
			clonedmut.set_attr(attr, value);
		}

		Ok(cloned)
	}
}

pub mod funcs {
	use super::*;
	use crate::value::ToAny;

	pub fn call(block: Gc<Block>, args: Args<'_>) -> Result<AnyValue> {
		block.run(args)
	}

	pub fn dbg(block: Gc<Block>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;

		let blockref = block.as_ref()?;

		// TODO: maybe cache this in the future?
		let mut builder = Text::simple_builder();
		builder.push_str("<Block:");
		builder.push_str(&blockref.inner().loc.to_string());
		builder.push('>');

		Ok(builder.finish().to_any())
	}
}

quest_type_attrs! { for Gc<Block>, parent Object;
	op_call => meth funcs::call,
	dbg => meth funcs::dbg,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
