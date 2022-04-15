use crate::{AnyValue, Result, Error};
use crate::vm::{Args, Frame};
use crate::value::{Gc, AsAny, Intern};
use crate::value::ty::Text;
use crate::value::base::Flags;
use std::sync::Arc;
use super::bytecode::Opcode;
use super::frame::FrameInner;
use std::cell::UnsafeCell;

/*

*/
quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Scope(Inner);
}

#[derive(Debug)]
pub struct Inner {
	frame: Arc<FrameInner>,
	pos: usize,
	unnamed_locals: Vec<Option<AnyValue>>,
	named_locals: Vec<Option<AnyValue>>,
}

const FLAG_CURRENTLY_RUNNING: u32 = Flags::USER0;

impl Scope {
	pub fn new(frame: Gc<Frame>, args: Args) -> Result<Gc<Scope>> {
		let frame = frame.as_ref()?.inner();
		let _ = args; // todo: use args

		let inner = Inner {
			unnamed_locals: vec![None; frame.num_of_unnamed_locals],
			named_locals: vec![None; frame.named_locals.len()],
			frame,
			pos: 0,
		};

		let scope = Gc::from_inner(crate::value::base::Base::new(inner, AnyValue::default()));

		Ok(scope)
	}
}

impl Gc<Scope> {
	// We define `run` on `Gc<Scope>` directly, because we need people to be able to mutably access
	// fields on us whilst we're running. 
	pub fn run(self) -> Result<AnyValue> {
		// If we're either currently mutably borrowed, or currently running, we cant actually run.
		// if !self.as_ref().and_then(|r| r.flags().try_acquire_all_user(FLAG_CURRENTLY_RUNNING)).unwrap_or(false) {
		// 	return Err("stackframe is currently running".to_string());
		// }

		todo!()

		// let did_return = self.as_ref().expect("unable to mark stackframe as not running?")
	}
}

// const LOCAL_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = 0xff;
