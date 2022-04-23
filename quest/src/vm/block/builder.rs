use super::Block;
use crate::value::{ty::Text, AnyValue, Gc};
use crate::vm::{bytecode::MAX_ARGUMENTS_FOR_SIMPLE_CALL, Opcode, SourceLocation};

pub struct Builder {
	loc: SourceLocation,
	code: Vec<u8>,
	constants: Vec<AnyValue>,
	num_of_unnamed_locals: usize,
	named_locals: Vec<Gc<Text>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Local {
	Scratch,
	Unnamed(usize),
	Named(usize),
}

const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;

impl Builder {
	#[must_use]
	pub fn new(loc: SourceLocation) -> Self {
		Self {
			loc,
			code: Vec::default(),
			constants: Vec::default(),
			num_of_unnamed_locals: 1, // The first register is scratch
			named_locals: Vec::default(),
		}
	}

	pub fn scratch(&self) -> Local {
		Local::Scratch
	}

	pub fn unnamed_local(&mut self) -> Local {
		self.num_of_unnamed_locals += 1;
		Local::Unnamed(self.num_of_unnamed_locals - 1)
	}

	pub fn named_local(&mut self, name: &str) -> Local {
		for (i, named_local) in self.named_locals.iter().enumerate() {
			// We created the `Gc<Text>` so no one else should be able to mutate them rn.
			if *named_local.as_ref().unwrap() == name {
				return Local::Named(i);
			}
		}

		self.named_locals.push(Text::from_str(name));
		Local::Named(self.named_locals.len() - 1)
	}

	#[must_use]
	pub fn build(self) -> Gc<Block> {
		Block::_new(
			self.code,
			self.loc,
			self.constants,
			self.num_of_unnamed_locals,
			self.named_locals,
		)
	}

	// SAFETY: you gotta make sure the remainder of the code after this is valid.
	unsafe fn opcode(&mut self, opcode: Opcode) {
		self.code.push(opcode as u8);
	}

	unsafe fn local(&mut self, local: Local) {
		match local {
			Local::Scratch => self.code.push(0),
			Local::Unnamed(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				self.code.push(n as u8)
			},
			Local::Unnamed(n) => {
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend(n.to_ne_bytes());
			},
			// todo, im not sure if this is 100% correct, math-wise
			Local::Named(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				self.code.push(!(n as i8) as u8)
			},
			Local::Named(n) => {
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend((!(n as isize)).to_ne_bytes());
			},
		}
	}

	unsafe fn count(&mut self, amnt: usize) {
		if amnt <= (u8::MAX as usize) {
			self.code.push(amnt as u8);
		} else {
			self.code.push(u8::MAX);
			self.code.extend(amnt.to_ne_bytes());
		}
	}

	#[inline]
	unsafe fn simple_opcode(&mut self, op: Opcode, args: &[Local]) {
		self.opcode(op);

		for arg in args {
			self.local(*arg);
		}
	}

	pub fn no_op(&mut self) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::NoOp, &[]);
		}
		self
	}

	pub fn debug(&mut self) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Mov, &[]);
		}
		self
	}

	pub fn mov(&mut self, from: Local, to: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Mov, &[from, to]);
		}
		self
	}

	pub fn call(&mut self) -> &mut Self {
		unsafe {
			self.opcode(Opcode::Call);
			todo!();
		}
		self
	}

	pub fn call_simple(&mut self, what: Local, args: &[Local], dst: Local) -> &mut Self {
		assert!(
			args.len() <= MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments given for call_simple: {}, max {}",
			args.len(),
			MAX_ARGUMENTS_FOR_SIMPLE_CALL
		);

		unsafe {
			self.opcode(Opcode::CallSimple);
			self.local(what);
			self.count(args.len());
			for arg in args {
				self.local(*arg);
			}
			self.local(dst);
		}
		self
	}

	pub fn constant(&mut self, value: AnyValue, dst: Local) -> &mut Self {
		let mut index = None;

		for (i, constant) in self.constants.iter().enumerate() {
			if constant.is_identical(value) {
				index = Some(i);
				break;
			}
		}

		let index = index.unwrap_or_else(|| {
			self.constants.push(value);
			self.constants.len() - 1
		});

		unsafe {
			self.opcode(Opcode::ConstLoad);
			self.count(index);
			self.local(dst);
		}
		self
	}

	pub fn current_frame(&mut self, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::CurrentFrame, &[dst]);
		}
		self
	}

	pub fn get_attr(&mut self, obj: Local, attr: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::GetAttr, &[obj, attr, dst]);
		}
		self
	}

	pub fn has_attr(&mut self, obj: Local, attr: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::HasAttr, &[obj, attr, dst]);
		}
		self
	}

	pub fn set_attr(&mut self, obj: Local, attr: Local, value: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::SetAttr, &[obj, attr, value]);
		}
		self
	}

	pub fn del_attr(&mut self, obj: Local, attr: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::DelAttr, &[obj, attr, dst]);
		}
		self
	}

	pub fn call_attr(&mut self) -> &mut Self {
		unsafe {
			self.opcode(Opcode::CallAttr);
		}
		todo!();
	}

	pub fn call_attr_simple(
		&mut self,
		obj: Local,
		attr: Local,
		args: &[Local],
		dst: Local,
	) -> &mut Self {
		assert!(
			args.len() <= MAX_ARGUMENTS_FOR_SIMPLE_CALL,
			"too many arguments given for call_attr_simple: {}, max {}",
			args.len(),
			MAX_ARGUMENTS_FOR_SIMPLE_CALL
		);

		unsafe {
			self.opcode(Opcode::CallAttrSimple);
			self.local(obj);
			self.local(attr);
			self.count(args.len());
			for arg in args {
				self.local(*arg);
			}
			self.local(dst);
		}
		self
	}

	pub fn add(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Add, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn subtract(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Subtract, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn multuply(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Multuply, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn divide(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Divide, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn modulo(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Modulo, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn power(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Power, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn not(&mut self, lhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Not, &[lhs, dst]);
		}
		self
	}

	pub fn negate(&mut self, lhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Negate, &[lhs, dst]);
		}
		self
	}

	pub fn equal(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Equal, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn notequal(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::NotEqual, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn less_than(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::LessThan, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn greater_than(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::GreaterThan, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn less_equal(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::LessEqual, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn greater_equal(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::GreaterEqual, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn compare(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Compare, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn index(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::Index, &[lhs, rhs, dst]);
		}
		self
	}

	pub fn index_assign(&mut self, lhs: Local, rhs: Local, dst: Local) -> &mut Self {
		unsafe {
			self.simple_opcode(Opcode::IndexAssign, &[lhs, rhs, dst]);
		}
		self
	}
}
