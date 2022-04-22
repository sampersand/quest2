use crate::{AnyValue, Result, Error};
use crate::vm::{Args, Frame};
use crate::value::{Gc, AsAny, Intern};
use crate::value::ty::Text;
use crate::value::base::Flags;
use std::sync::Arc;
use super::bytecode::Opcode;
use super::frame::InnerFrame;
use std::cell::UnsafeCell;

/*

*/
quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Scope(UnsafeCell<Inner>);
}

pub struct NotCurrentlyReferenced(Gc<Scope>);

/*
*/
#[derive(Debug)]
pub struct Inner {
	gc_frame: Gc<Frame>,
	frame: Arc<InnerFrame>,
	pos: usize,
	unnamed_locals: Vec<Option<AnyValue>>,
	named_locals: Vec<Option<AnyValue>>,
}


/*
Stackframes, if theyve never been directly referenced in quest code before, will have named variables
within `named_locals`. Once you named them, they all transition over into the header like any other
object. (You can tell bc `named_locals` is empty, although we might want to just set a flag).

Attempting to run a stackframe when it's currently being run is an error, so this can be indicated
via a user flag.
*/

const LOCAL_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = 0xff;
const FLAG_INNER_ACQUIRED: u32 = Flags::USER0;

// impl Scope {
// 	fn inner_mut(&self) -> Option<&mut Inner> {
// 		if self.0.header().flags().try_acquire_all_user(FLAG_INNER_ACQUIRED) {
// 			// SAFETY: by the atomicity of the `try_acquire_all_user` function, we know we are
// 			// the only ones who have access to this data.
// 			Some(unsafe { &mut *self.0.data().get() })
// 		} else {
// 			None
// 		}
// 	}

// 	fn convert_locals_to_attributes(&mut self) -> Result<()> {
// 		todo!();
// 		// let (header, data) = self.0.header_data_mut();
// 		// // OPTIMIZE: we could use `with_capacity`, but we'd need to move it out of the builder.

// 		// for (value, name) in data.named_locals.drain(..).zip(data.frame.named_locals.iter()) {
// 		// 	// Only insert named and assigned values.
// 		// 	if let Some(value) = value {
// 		// 		if let Some(intern) = Intern::from_str(name) {
// 		// 			header.set_attr(intern, value)?;
// 		// 		} else {
// 		// 			header.set_attr(Text::from_str(name).as_any(), value)?;
// 		// 		}
// 		// 	}
// 		// }

// 		// Ok(())
// 	}

// 	fn get_named_local(&self, local: usize) -> Result<Option<AnyValue>> {
// 		// If we haven't converted `self` to an object, first check the local.
// 		if !self.named_locals.is_empty() {
// 			// If it's defined locally, return that.
// 			if let Some(attr) = self.named_locals[local] {
// 				return Ok(Some(attr));
// 			}
// 		}

// 		// OPTIMIZE: do we need to allocate a new `Text` every time? it'll be slow...
// 		let name = Text::from_str(&self.frame.named_locals[local]).as_any();

// 		if self.named_locals.is_empty() {
// 			// Dont search the parents cause we do next.
// 			if let Some(attr) = self.0.header().get_unbound_attr(name, false)? {
// 				return Ok(Some(attr));
// 			}
// 			// We search the parents because youre allowed to remove the attr from the child.
// 		}

// 		// We still might be able to get it from the parents, eg from enclosing scopes.
// 		self.0.header().get_unbound_attr_from_parents(name)
// 	}

// 	fn get_local(&self, local: isize) -> Result<AnyValue> {
// 		if let Ok(unnamed_local) = usize::try_from(local) {
// 			self.unnamed_locals[unnamed_local]
// 		} else {
// 			self.get_named_local(!local as usize)?
// 		}.ok_or_else(|| panic!("todo: access undefined local variable"))
// 	}


// 	fn set_local(&mut self, local: isize, value: AnyValue) -> Result<()> {
// 		if let Ok(unnamed_local) = usize::try_from(local) {
// 			self.unnamed_locals[unnamed_local] = Some(value);
// 			return Ok(())
// 		}

// 		let local = !local as usize;

// 		// If we haven't converted `self` to an object, first check the local.
// 		if !self.named_locals.is_empty() {
// 			self.named_locals[local] = Some(value);
// 			return Ok(());
// 		}

// 		// TODO: maybe try intern too
// 		let name = Text::from_str(&self.frame.named_locals[local]).as_any();

// 		self.0.header_mut().set_attr(name, value)
// 	}
// }

// impl Scope {
// 	pub fn new(gc_frame: Gc<Frame>, args: Args) -> Gc<Self> {
// 		let _ = args; // todo: use args

// 		let frame = gc_frame.as_ref().expect("frame is currently borrowed?").0.data().clone();
// 		let unnamed_locals = vec![None; frame.num_of_unnamed_locals];
// 		let named_locals = vec![None; frame.named_locals.len()];

// 		Gc::from_inner(crate::value::base::Base::new(Inner {
// 			gc_frame,
// 			frame,
// 			pos: 0,
// 			named_locals,
// 			unnamed_locals,
// 		}, AnyValue::default()))
// 	}
// }

// impl Scope {
// 	fn next_byte(&mut self) -> Option<u8> {
// 		let byte = *self.frame.code.get(self.pos)?;
// 		self.pos += 1;
// 		Some(byte)
// 	}

// 	fn next_usize(&mut self) -> Option<usize> {
// 		const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();

// 		let slice = self.frame.code.get(self.pos..self.pos + SIZEOF_USIZE)?;
// 		let us = usize::from_ne_bytes(slice.try_into().unwrap());

// 		self.pos += SIZEOF_USIZE;

// 		Some(us)
// 	}

// 	fn next_local(&mut self) -> Result<AnyValue> {
// 		let amnt = self.next_count() as isize;
// 		self.get_local(amnt)
// 	}

// 	fn next_count(&mut self) -> usize {
// 		match self.next_byte().expect("missing byte for local") {
// 			LOCAL_IS_NOT_ONE_BYTE_BUT_USIZE => self.next_usize().expect("missing usize for local"),
// 			other => other as usize
// 		}
// 	}

// 	fn next_opcode(&mut self) -> Option<Opcode> {
// 		Some(match self.next_byte()? {
// 			op if op == Opcode::NoOp as u8 => Opcode::NoOp,
// 			op if op == Opcode::Debug as u8 => Opcode::Debug,

// 			op if op == Opcode::Mov as u8 => Opcode::Mov,
// 			op if op == Opcode::Call as u8 => Opcode::Call,
// 			op if op == Opcode::Return as u8 => Opcode::Return,

// 			op if op == Opcode::ConstLoad as u8 => Opcode::ConstLoad,
// 			op if op == Opcode::GetAttr as u8 => Opcode::GetAttr,
// 			op if op == Opcode::HasAttr as u8 => Opcode::HasAttr,
// 			op if op == Opcode::SetAttr as u8 => Opcode::SetAttr,
// 			op if op == Opcode::DelAttr as u8 => Opcode::DelAttr,
// 			op if op == Opcode::CallAttr as u8 => Opcode::CallAttr,

// 			op if op == Opcode::Not as u8 => Opcode::Not,
// 			op if op == Opcode::Negate as u8 => Opcode::Negate,
// 			op if op == Opcode::Equal as u8 => Opcode::Equal,
// 			op if op == Opcode::NotEqual as u8 => Opcode::NotEqual,
// 			op if op == Opcode::LessThan as u8 => Opcode::LessThan,
// 			op if op == Opcode::GreaterThan as u8 => Opcode::GreaterThan,
// 			op if op == Opcode::LessEqual as u8 => Opcode::LessEqual,
// 			op if op == Opcode::GreaterEqual as u8 => Opcode::GreaterEqual,
// 			op if op == Opcode::Compare as u8 => Opcode::Compare,
// 			op if op == Opcode::Add as u8 => Opcode::Add,
// 			op if op == Opcode::Subtract as u8 => Opcode::Subtract,
// 			op if op == Opcode::Multuply as u8 => Opcode::Multuply,
// 			op if op == Opcode::Divide as u8 => Opcode::Divide,
// 			op if op == Opcode::Modulo as u8 => Opcode::Modulo,
// 			op if op == Opcode::Power as u8 => Opcode::Power,
// 			op if op == Opcode::Index as u8 => Opcode::Index,
// 			op if op == Opcode::IndexAssign as u8 => Opcode::IndexAssign,
// 			other => panic!("unknown opcode {:02x}", other)
// 		})
// 	}

// 	fn constant(&self, index: usize) -> AnyValue {
// 		self.frame.constants[index]
// 	}

// 	fn store_next_local(&mut self, value: AnyValue) {
// 		let index = self.next_count();
// 		self.set_local(index as isize, value);
// 	}
// }


// impl Gc<Scope> {
// 	fn op_noop(&self) {}

// 	fn op_debug(&self) {
// 		dbg!(self.as_ref().unwrap());
// 	}

// 	fn op_mov(&self) -> Result<()> {
// 		let mut this = self.as_mut()?;
// 		let src = this.next_local()?;
// 		this.store_next_local(src);
// 		Ok(())
// 	}

// 	fn op_return(&self) -> Result<AnyValue> {
// 		let mut this = self.as_mut()?;
// 		Err(Error::Return {
// 			value: this.next_local()?,
// 			from_frame: this.next_local()?
// 		})
// 	}

// 	fn op_constload(&self) -> Result<()> {
// 		let mut this = self.as_mut()?;
// 		let idx = this.next_count();
// 		let constant = this.constant(idx);
// 		this.store_next_local(constant);
// 		Ok(())
// 	}

// 	fn op_getattr(&self) -> Result<()>{
// 		let mut this = self.as_mut()?;
// 		let object = this.next_local();
// 		let attr = this.next_local();

// 		drop(this); // as `get_attr` may modify us.
// 		let value = object.get_attr(attr)

// 		Ok();
// 	}
// 	fn op_hasattr(&self) -> Result<()>{
// 		let mut this = self.as_mut()?;

// 		Ok();
// 	}
// 	fn op_setattr(&self) -> Result<()>{
// 		let mut this = self.as_mut()?;

// 		Ok();
// 	}
// 	fn op_delattr(&self) -> Result<()>{
// 		let mut this = self.as_mut()?;

// 		Ok();
// 	}
// 	fn op_callattr(&self) -> Result<()>{
// 		let mut this = self.as_mut()?;

// 		Ok();
// 	}

// 	pub fn run(self) -> Result<AnyValue> {
// 		loop {
// 			let op = if let Some(opcode) = self.as_mut()?.next_opcode() {
// 				opcode
// 			} else {
// 				break;
// 			};

// 			match op {
// 				Opcode::NoOp => self.op_noop(),
// 				Opcode::Debug => self.op_debug(),
// 				Opcode::Mov => self.op_mov()?,
// 				Opcode::Call => todo!("call"),
// 				Opcode::Return => return self.op_return(),
// 				Opcode::ConstLoad => self.op_constload()?,
// 				Opcode::GetAttr => self.op_getattr()?,
// 				Opcode::HasAttr => self.op_hasattr()?,
// 				Opcode::SetAttr => self.op_setattr()?,
// 				Opcode::DelAttr => self.op_delattr()?,
// 				Opcode::CallAttr => self.op_callattr()?,
// 				_ => todo!()
// 			}
// 		}

// 		Ok(AnyValue::default())
// 	}
// }
