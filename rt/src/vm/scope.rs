use crate::{AnyValue, Result, Error};
use crate::vm::{Args, Frame};
use crate::value::Gc;
use super::bytecode::Opcode;


quest_type! {
	#[derive(NamedType)]
	pub struct Scope(Inner);
}

#[derive(Debug)]
struct Inner {
	frame: Gc<Frame>,
	pos: usize,
	locals: Vec<AnyValue>
}

const LOCAL_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = 0xff;

impl Scope {
	pub fn new(frame: Gc<Frame>, args: Args) -> Gc<Self> {
		let locals = vec![AnyValue::default(); 1/*frame.as_ref().unwrap().nlocals_total*/];
		let _ = args; // todo: use args

		Gc::from_inner(crate::value::base::Base::new(Inner { frame, pos: 0, locals }, AnyValue::default()))
	}
}

impl Inner {
	pub fn next_byte(&mut self) -> Option<u8> {
		let byte = self.frame.code.get(self.pos)?;
		self.pos += 1;
		Some(*byte)
	}

	pub fn next_usize(&mut self) -> Option<usize> {
		let slice = self.frame.code.get(self.pos..self.pos + std::mem::size_of::<usize>())?;

		self.pos += std::mem::size_of::<usize>();

		Some(usize::from_ne_bytes(slice.try_into().unwrap()))
	}

	pub fn next_local(&mut self) -> AnyValue {
		let count = self.next_count();
		self.locals[count]
	}

	pub fn next_count(&mut self) -> usize {
		match self.next_byte().expect("missing byte for local") {
			LOCAL_IS_NOT_ONE_BYTE_BUT_USIZE => self.next_usize().expect("missing usize for local"),
			other => other as usize
		}
	}

	pub fn next_opcode(&mut self) -> Option<Opcode> {
		Some(match self.next_byte()? {
			op if op == Opcode::NoOp as u8 => Opcode::NoOp,

			op if op == Opcode::Mov as u8 => Opcode::Mov,
			op if op == Opcode::Call as u8 => Opcode::Call,
			op if op == Opcode::Return as u8 => Opcode::Return,

			op if op == Opcode::ConstLoad as u8 => Opcode::ConstLoad,
			op if op == Opcode::GetAttr as u8 => Opcode::GetAttr,
			op if op == Opcode::HasAttr as u8 => Opcode::HasAttr,
			op if op == Opcode::SetAttr as u8 => Opcode::SetAttr,
			op if op == Opcode::DelAttr as u8 => Opcode::DelAttr,
			op if op == Opcode::CallAttr as u8 => Opcode::CallAttr,

			op if op == Opcode::Not as u8 => Opcode::Not,
			op if op == Opcode::Negate as u8 => Opcode::Negate,
			op if op == Opcode::Equal as u8 => Opcode::Equal,
			op if op == Opcode::NotEqual as u8 => Opcode::NotEqual,
			op if op == Opcode::LessThan as u8 => Opcode::LessThan,
			op if op == Opcode::GreaterThan as u8 => Opcode::GreaterThan,
			op if op == Opcode::LessEqual as u8 => Opcode::LessEqual,
			op if op == Opcode::GreaterEqual as u8 => Opcode::GreaterEqual,
			op if op == Opcode::Compare as u8 => Opcode::Compare,
			op if op == Opcode::Add as u8 => Opcode::Add,
			op if op == Opcode::Subtract as u8 => Opcode::Subtract,
			op if op == Opcode::Multuply as u8 => Opcode::Multuply,
			op if op == Opcode::Divide as u8 => Opcode::Divide,
			op if op == Opcode::Modulo as u8 => Opcode::Modulo,
			op if op == Opcode::Power as u8 => Opcode::Power,
			op if op == Opcode::Index as u8 => Opcode::Index,
			op if op == Opcode::IndexAssign as u8 => Opcode::IndexAssign,
			other => panic!("unknown opcode {:02x}", other)
		})
	}

	pub fn local(&self, local: usize) -> AnyValue {
		self.locals[local]
	}

	pub fn local_mut(&mut self, local: usize) -> &mut AnyValue {
		&mut self.locals[local]
	}

	pub fn constant(&self, index: usize) -> AnyValue {
		self.frame.constants[index]
	}
}

impl Scope {
	fn run(mut self) -> Result<AnyValue> {
		while let Some(opcode) = self.next_opcode() {
			self.run_opcode(opcode)?;
			dbg!(&self);
		}

		Ok(AnyValue::default())
	}

	fn store_next_local(&mut self, value: AnyValue) {
		let index = self.next_count();
		*self.local_mut(index) = value;
	}

	fn run_opcode(&mut self, opcode: Opcode) -> Result<()> {
		match opcode {
			Opcode::NoOp => {},
			Opcode::Mov => {
				let src = self.next_local();
				self.store_next_local(src);
			},
			Opcode::Return => return Err(Error::Return {
				value: self.next_local(),
				from_frame: self.next_local()
			}),
			Opcode::ConstLoad => {
				let idx = self.next_count();
				self.store_next_local(self.constant(idx));
			},
			_ => todo!()
		}

		Ok(())
	}
}
