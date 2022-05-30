use super::Block;
use crate::value::ToAny;
use crate::value::{ty::Text, AnyValue, Gc};
use crate::vm::{bytecode::MAX_ARGUMENTS_FOR_SIMPLE_CALL, Opcode, SourceLocation};

#[derive(Debug)]
pub struct Builder {
	loc: SourceLocation,
	code: Vec<u8>,
	constants: Vec<AnyValue>,
	num_of_unnamed_locals: usize,
	named_locals: Vec<Gc<Text>>,
	parent_scope: Option<AnyValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Local {
	Scratch,
	Unnamed(usize),
	Named(usize),
}

const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;

impl Default for Builder {
	fn default() -> Self {
		Self::new(SourceLocation::default(), None)
	}
}

impl Builder {
	#[must_use]
	pub fn new(loc: SourceLocation, parent_scope: Option<AnyValue>) -> Self {
		// these are present in every block
		let named_locals = vec![
			Text::from_static_str("__block__"),
			Text::from_static_str("__args__"),
		];

		Self {
			loc,
			code: Vec::default(),
			constants: Vec::default(),
			num_of_unnamed_locals: 1, // The first register is scratch
			named_locals,
			parent_scope,
		}
	}

	pub fn unnamed_local(&mut self) -> Local {
		self.num_of_unnamed_locals += 1;
		Local::Unnamed(self.num_of_unnamed_locals - 1)
	}

	pub fn named_local(&mut self, name: &str) -> Local {
		for (idx, named_local) in self.named_locals.iter().enumerate() {
			// We created the `Gc<Text>` so no one else should be able to mutate them rn.
			if *named_local.as_ref().unwrap() == name {
				trace!(target: "block_builder", ?idx, ?name, "found named local");
				return Local::Named(idx);
			}
		}

		let idx = self.named_locals.len();
		trace!(target: "block_builder", ?idx, ?name, "created new local");

		self.named_locals.push(Text::from_str(name));
		Local::Named(idx)
	}

	#[must_use]
	pub fn build(self) -> Gc<Block> {
		Block::_new(
			self.code,
			self.loc,
			self.constants,
			self.num_of_unnamed_locals,
			self.named_locals,
			self.parent_scope,
		)
	}

	pub fn str_constant(&mut self, string: &str, dst: Local) {
		let mut index = None;

		for (idx, constant) in self.constants.iter().enumerate() {
			if let Some(text) = constant.downcast::<Gc<Text>>() {
				if *text.as_ref().unwrap() == string {
					trace!(target: "block_builder", ?idx, ?string, "found str constant");
					index = Some(idx);
					break;
				}
			}
		}

		let index = index.unwrap_or_else(|| {
			let idx = self.constants.len();
			trace!(target: "block_builder", ?idx, ?string, "created str constant");

			self.constants.push(Text::from_str(string).to_any());
			idx
		});

		unsafe {
			self.opcode(Opcode::ConstLoad, dst);
			self.count(index);
		}
	}

	pub fn constant(&mut self, value: AnyValue, dst: Local) {
		let mut index = None;

		for (idx, constant) in self.constants.iter().enumerate() {
			if constant.is_identical(value) {
				trace!(target: "block_builder", ?idx, ?value, "found constant");
				index = Some(idx);
				break;
			}
		}

		let index = index.unwrap_or_else(|| {
			let idx = self.constants.len();
			trace!(target: "block_builder", ?idx, ?value, "created constant");

			self.constants.push(value);
			idx
		});

		unsafe {
			self.opcode(Opcode::ConstLoad, dst);
			self.count(index);
		}
	}

	// SAFETY: you gotta make sure the remainder of the code after this is valid.
	unsafe fn opcode(&mut self, opcode: Opcode, dst: Local) {
		debug!(target: "block_builder", idx=self.code.len(), ?opcode, "set byte");
		self.code.push(opcode as u8);
		self.local(dst);
	}

	unsafe fn local(&mut self, local: Local) {
		// todo: local
		// debug!(target: "block_builder", "self[{}].local = 0 (scratch)", self.code.len());
		match local {
			Local::Scratch => {
				debug!(target: "block_builder", idx=self.code.len(), local=%"0 (scratch)", "set byte");
				self.code.push(0);
			},
			Local::Unnamed(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				debug!(target: "block_builder", idx=self.code.len(), local=%n, "set byte");
				self.code.push(n as u8);
			},
			Local::Unnamed(n) => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, "set bytes");
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend(n.to_ne_bytes());
			},
			// todo, im not sure if this is 100% correct, math-wise
			Local::Named(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, updated=?(!(n as i8) as u8), "set byte");
				self.code.push(!(n as i8) as u8);
			},
			Local::Named(n) => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, updated=?((!n as isize) as usize), "set bytes");
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend((!(n as isize)).to_ne_bytes());
			},
		}
	}

	unsafe fn count(&mut self, count: usize) {
		use crate::vm::bytecode::COUNT_IS_NOT_ONE_BYTE_BUT_USIZE;

		// TODO: verify this is sound.
		if count <= COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize {
			debug!(target: "block_builder", idx=self.code.len(), ?count, "set byte");
			self.code.push(count as u8);
		} else {
			debug!(target: "block_builder", idx=self.code.len(), ?count, "set bytes");
			self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
			self.code.extend(count.to_ne_bytes());
		}
	}

	#[inline]
	pub unsafe fn simple_opcode(&mut self, op: Opcode, dst: Local, args: &[Local]) {
		self.opcode(op, dst);

		for arg in args {
			self.local(*arg);
		}
	}

	pub fn mov(&mut self, from: Local, to: Local) {
		if from == to {
			return;
		}

		unsafe {
			self.simple_opcode(Opcode::Mov, to, &[from]);
		}
	}

	pub fn create_list(&mut self, args: &[Local], dst: Local) {
		unsafe {
			self.opcode(Opcode::CreateList, dst);
			self.count(args.len());
			for arg in args {
				self.local(*arg);
			}
		}
	}

	pub fn call(&mut self, dst: Local) {
		unsafe {
			self.opcode(Opcode::Call, dst);
			todo!();
		}
	}

	pub fn call_simple(&mut self, what: Local, args: &[Local], dst: Local) {
		assert!(
			args.len() <= MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments given for call_simple: {}, max {}",
			args.len(),
			MAX_ARGUMENTS_FOR_SIMPLE_CALL
		);

		unsafe {
			self.opcode(Opcode::CallSimple, dst);
			self.local(what);
			self.count(args.len());
			for arg in args {
				self.local(*arg);
			}
		}
	}

	pub fn stackframe(&mut self, depth: isize, dst: Local) {
		unsafe {
			self.opcode(Opcode::Stackframe, dst);
			self.count(depth as usize);
		}
	}

	pub fn get_unbound_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::GetUnboundAttr, dst, &[obj, attr]);
		}
	}

	pub fn get_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::GetAttr, dst, &[obj, attr]);
		}
	}

	pub fn has_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::HasAttr, dst, &[obj, attr]);
		}
	}

	pub fn set_attr(&mut self, obj: Local, attr: Local, value: Local, dst: Local) {
		// NOTE: this puts `obj` last as that allows for optimizations on `attr` and `value` parsing
		unsafe {
			self.simple_opcode(Opcode::SetAttr, dst, &[attr, value, obj]);
		}
	}

	pub fn del_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::DelAttr, dst, &[obj, attr]);
		}
	}

	pub fn call_attr(&mut self, dst: Local) {
		unsafe {
			self.opcode(Opcode::CallAttr, dst);
		}
		todo!();
	}

	pub fn call_attr_simple(&mut self, obj: Local, attr: Local, args: &[Local], dst: Local) {
		assert!(
			args.len() <= MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments given for call_attr_simple: {}, max {}",
			args.len(),
			MAX_ARGUMENTS_FOR_SIMPLE_CALL
		);

		unsafe {
			self.opcode(Opcode::CallAttrSimple, dst);
			self.local(obj);
			self.local(attr);
			self.count(args.len());
			for arg in args {
				self.local(*arg);
			}
		}
	}

	pub fn add(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Add, dst, &[lhs, rhs]);
		}
	}

	pub fn subtract(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Subtract, dst, &[lhs, rhs]);
		}
	}

	pub fn multiply(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Multiply, dst, &[lhs, rhs]);
		}
	}

	pub fn divide(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Divide, dst, &[lhs, rhs]);
		}
	}

	pub fn modulo(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Modulo, dst, &[lhs, rhs]);
		}
	}

	pub fn power(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Power, dst, &[lhs, rhs]);
		}
	}

	pub fn not(&mut self, lhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Not, dst, &[lhs]);
		}
	}

	pub fn negate(&mut self, lhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Negate, dst, &[lhs]);
		}
	}

	pub fn equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Equal, dst, &[lhs, rhs]);
		}
	}

	pub fn notequal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::NotEqual, dst, &[lhs, rhs]);
		}
	}

	pub fn less_than(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::LessThan, dst, &[lhs, rhs]);
		}
	}

	pub fn greater_than(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::GreaterThan, dst, &[lhs, rhs]);
		}
	}

	pub fn less_equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::LessEqual, dst, &[lhs, rhs]);
		}
	}

	pub fn greater_equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::GreaterEqual, dst, &[lhs, rhs]);
		}
	}

	pub fn compare(&mut self, lhs: Local, rhs: Local, dst: Local) {
		unsafe {
			self.simple_opcode(Opcode::Compare, dst, &[lhs, rhs]);
		}
	}

	pub fn index(&mut self, source: Local, index: &[Local], dst: Local) {
		assert!(
			index.len() <= MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments for index ({} max, got {}), use call_attr instead",
			MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			index.len(),
		);

		unsafe {
			self.opcode(Opcode::Index, dst);
			self.local(source);
			self.count(index.len());
			for arg in index {
				self.local(*arg);
			}
		}
	}

	pub fn index_assign(&mut self, source: Local, index: &[Local], value: Local, dst: Local) {
		assert!(
			index.len() < MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments for index_assign ({} max, got {}), use call_attr instead",
			MAX_ARGUMENTS_FOR_SIMPLE_CALL - 1, // `-1` as `value` is the last one
			index.len(),
		);
		unsafe {
			self.opcode(Opcode::IndexAssign, dst);
			self.local(source);
			self.count(index.len() + 1);
			for arg in index {
				self.local(*arg);
			}
			self.local(value);
		}
	}
}
