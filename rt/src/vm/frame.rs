use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use crate::{AnyValue, Result};
use crate::value::{Gc, HasDefaultParent};
use crate::vm::Args;
use std::sync::Arc;
use super::{Scope, SourceLocation};

quest_type! {
	#[derive(NamedType, Debug)]
	pub struct Frame(Arc<FrameInner>);
}

#[derive(Debug)]
pub struct FrameInner {
	pub(super) frame: UnsafeCell<MaybeUninit<Gc<Frame>>>,
	pub(super) code: Vec<u8>,
	pub(super) loc: SourceLocation,
	pub(super) constants: Vec<AnyValue>,
	pub(super) num_of_unnamed_locals: usize,
	pub(super) named_locals: Vec<String>,
}

impl Frame {
	pub fn _new(code: Vec<u8>, loc: SourceLocation, constants: Vec<AnyValue>,
		num_of_unnamed_locals: usize, named_locals: Vec<String>
	) -> Gc<Self> {
		let inner = Arc::new(FrameInner {
			frame: UnsafeCell::new(MaybeUninit::uninit()),
			code, loc, constants, num_of_unnamed_locals, named_locals
		});
		let gc = Gc::from_inner(crate::value::base::Base::new(inner.clone(), Gc::<Frame>::parent()));
		unsafe {
			inner.frame.get().write(MaybeUninit::new(gc));
		}
		gc
	}

	pub(crate) fn inner(&self) -> Arc<FrameInner> {
		self.0.data().clone()
	}
}

impl Gc<Frame> {
	pub fn run(self, args: Args) -> Result<AnyValue> {
		Scope::new(self, args)?.run()
	}
}

quest_type_attrs! { for Gc<Frame>;
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
