use crate::{AnyValue, Result};
use crate::value::Gc;
use super::Scope;

quest_type! {
	#[derive(NamedType, Debug)]
	pub struct Frame(Arc<Inner>);
}

#[derive(Debug)]
struct Inner {
	code: Vec<u8>,
	loc: SourceLocation,
	constants: Vec<AnyValue>,
	nlocals_total: usize,
	named_locals: Vec<(String, usize)>,
}

impl Frame {
	pub fn _new(code: Vec<u8>, loc: SourceLocation, constants: Vec<AnyValue>,
		nlocals_total: usize, named_locals: Vec<(String, usize)>) -> Gc<Self> {
		todo!()
		// Self { code, loc, constants, nlocals_total, named_locals }
	}
}

impl Gc<Frame> {
	pub fn run(self, args: Args) -> Result<AnyValue> {
		Scope::new(self, args).run()
	}
}
