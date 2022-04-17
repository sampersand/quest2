use crate::{AnyValue, Result, Error};
use crate::vm::{Args, Block};
use crate::value::{Gc, AsAny, Intern, HasDefaultParent};
use crate::value::ty::{Text, List};
use crate::value::base::{Flags, Base};
use std::sync::Arc;
use super::bytecode::Opcode;
use super::block::BlockInner;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Frame(Inner);
}

#[derive(Debug)]
pub struct Inner {
	block: Arc<BlockInner>,
	pos: usize,
	unnamed_locals: Vec<AnyValue>,
	named_locals: Vec<Option<AnyValue>>,
}

const FLAG_CURRENTLY_RUNNING: u32 = Flags::USER0;
const FLAG_IS_OBJECT: u32 = Flags::USER1;
const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;

impl Frame {
	pub fn new(gc_block: Gc<Block>, args: Args) -> Result<Gc<Frame>> {
		let block = gc_block.as_ref()?.inner();

		let mut named_locals = vec![None; block.named_locals.len()];
		args.assert_no_keyword()?; // todo: assign keyword arguments
		for (i, arg) in args.positional().iter().enumerate() {
			named_locals[i] = Some(*arg);
		}

		let inner = Inner {
			unnamed_locals: vec![Default::default(); block.num_of_unnamed_locals],
			named_locals,
			block,
			pos: 0,
		};

		let parents = List::from_slice(&[Gc::<Frame>::parent(), gc_block.as_any()]);
		let frame = Gc::from_inner(Base::new(inner, parents));

		Ok(frame)
	}

	fn is_object(&self) -> bool {
		self.0.header().flags().contains(FLAG_IS_OBJECT)
	}

	fn convert_to_object(&mut self) -> Result<()> {
		// If we're already an object, nothing else needed to be done.
		if !self.0.header().flags().try_acquire_all_user(FLAG_IS_OBJECT) {
			return Ok(())
		}

		let (header, data) = self.0.header_data_mut();
		// OPTIMIZE: we could use `with_capacity`, but we'd need to move it out of the builder.

		for (value, name) in data.named_locals.drain(..).zip(data.block.named_locals.iter()) {
			// Only insert named and assigned values.
			if let Some(value) = value {
				header.set_attr(name.as_any(), value)?;
			}
		}

		Ok(())
	}

	fn get_local(&self, index: isize) -> Result<AnyValue> {
		if let Ok(amnt) = usize::try_from(index) {
			return Ok(self.unnamed_locals[amnt]);
		}

		let index = !index as usize;

		if !self.is_object() {
			// Since we could be trying to access a parent scope's variable, we won't return an error
			// in the false case.
			if let Some(value) = self.named_locals[index] {
				return Ok(value);
			}
		}

		let attr_name = self.block.named_locals[index];
		self.0.header().get_unbound_attr(attr_name.as_any(), true)?
			.ok_or_else(|| format!("unknown attribute {:?}", attr_name).into())
	}

	fn set_local(&mut self, index: isize, value: AnyValue) -> Result<()> {
		if let Ok(index) = usize::try_from(index) {
			self.unnamed_locals[index] = value;
			return Ok(());
		}

		let index = !index as usize;

		if !self.is_object() {
			self.named_locals[index] = Some(value);
			return Ok(())
		}

		let attr = self.block.named_locals[index];
		self.0.header_mut().set_attr(attr.as_any(), value)
	}
}

impl Deref for Frame {
	type Target = Inner;

	fn deref(&self) -> &Self::Target {
		self.0.data()
	}
}

impl DerefMut for Frame {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.data_mut()
	}
}

impl Frame {
	fn next_byte(&mut self) -> Option<u8> {
		let byte = *self.block.code.get(self.pos)?;
		self.pos += 1;
		Some(byte)
	}

	fn next_usize(&mut self) -> Option<usize> {
		const SIZEOF_USIZE: usize = std::mem::size_of::<usize>();

		let slice = self.block.code.get(self.pos..self.pos + SIZEOF_USIZE)?;
		let us = usize::from_ne_bytes(slice.try_into().unwrap());

		self.pos += SIZEOF_USIZE;

		Some(us)
	}

	fn next_local(&mut self) -> Result<AnyValue> {
		let index = self.next_count() as isize;
		self.get_local(index)
	}

	fn store_next_local(&mut self, value: AnyValue) {
		let index = self.next_count() as isize;
		self.set_local(index, value);
	}

	fn next_count(&mut self) -> usize {
		let byte = self.next_byte().expect("missing byte for local");
		if byte == COUNT_IS_NOT_ONE_BYTE_BUT_USIZE {
			return self.next_usize().expect("missing usize for local");
		}

		if (byte as i8) < 0 {
			byte as i8 as isize as usize
		} else {
			byte as usize
		}
	}

	fn next_opcode(&mut self) -> Option<Opcode> {
		Some(match self.next_byte()? {
			op if op == Opcode::NoOp as u8 => Opcode::NoOp,
			op if op == Opcode::Debug as u8 => Opcode::Debug,

			op if op == Opcode::Mov as u8 => Opcode::Mov,
			op if op == Opcode::Call as u8 => Opcode::Call,
			op if op == Opcode::CallSimple as u8 => Opcode::CallSimple,
			op if op == Opcode::Return as u8 => Opcode::Return,

			op if op == Opcode::ConstLoad as u8 => Opcode::ConstLoad,
			op if op == Opcode::CurrentFrame as u8 => Opcode::CurrentFrame,
			op if op == Opcode::GetAttr as u8 => Opcode::GetAttr,
			op if op == Opcode::HasAttr as u8 => Opcode::HasAttr,
			op if op == Opcode::SetAttr as u8 => Opcode::SetAttr,
			op if op == Opcode::DelAttr as u8 => Opcode::DelAttr,
			op if op == Opcode::CallAttr as u8 => Opcode::CallAttr,
			op if op == Opcode::CallAttrSimple as u8 => Opcode::CallAttrSimple,

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
}


// impl Gc<Frame> {
// 	// We define `run` on `Gc<Frame>` directly, because we need people to be able to mutably access
// 	// fields on us whilst we're running. 
// 	pub fn run(self) -> Result<AnyValue> {
// 		// If we're either currently mutably borrowed, or currently running, we cant actually run.
// 		// if !self.as_ref().and_then(|r| r.flags().try_acquire_all_user(FLAG_CURRENTLY_RUNNING)).unwrap_or(false) {
// 		// 	return Err("stackframe is currently running".to_string());
// 		// }

// 		todo!()

// 		// let did_return = self.as_ref().expect("unable to mark stackframe as not running?")
// 	}
// }

impl Gc<Frame> {
	fn op_noop(&self) {}

	fn op_debug(&self) {
		dbg!(&self.as_ref().unwrap().unnamed_locals);
		// dbg!(&self.as_ref().unwrap().0.header().attributes());
		// dbg!(self.as_ref().unwrap().get_local(-2));
		// panic!();
	}

	fn op_mov(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let src = this.next_local()?;
		this.store_next_local(src);
		Ok(())
	}

	fn op_call_simple(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let amnt = this.next_count();
		let mut positional = Vec::with_capacity(amnt);

		for _ in 0..amnt {
			positional.push(this.next_local()?);
		}
		drop(this);
		let result = object.call(Args::new(&positional, &[]))?;
		self.as_mut()?.store_next_local(result);

		Ok(())
	}

	fn op_return(&self) -> Result<AnyValue> {
		let mut this = self.as_mut()?;
		Err(Error::Return {
			value: this.next_local()?,
			from_frame: this.next_local()?
		})
	}

	// TODO: we need to make this CoW, as otherwise this happens:
	// ```
	// foo = x -> { l = "A"; l.a = x; l };
	// q = foo(3);
	// foo(4);
	// print(q.a); #=> 4
	// ```
	fn op_constload(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let idx = this.next_count();
		let constant = this.block.constants[idx];
		this.store_next_local(constant);
		Ok(())
	}

	fn op_currentframe(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		this.convert_to_object()?;
		this.store_next_local(self.clone().as_any());
		Ok(())
	}

	fn op_getattr(&self) -> Result<()>{
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;

		drop(this); // as `get_attr` may modify us.
		let value = object.get_attr(attr)?.expect("todo: we should actually make this return a straight Result");
		self.as_mut()?.store_next_local(value);

		Ok(())
	}

	fn op_hasattr(&self) -> Result<()>{
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;

		drop(this); // as `has_attr` may modify us.
		let hasit = object.has_attr(attr)?;
		self.as_mut()?.store_next_local(hasit.as_any());

		Ok(())
	}
	
	fn op_setattr(&self) -> Result<()>{
		let mut this = self.as_mut()?;
		let object_index = this.next_count() as isize;
		let attr = this.next_local()?;
		let value = this.next_local()?;

		/*
		Because you can assign indices onto any object, we need to be able to dynamically convert
		immediates (eg integers, floats, booleans, etc) into a heap-allocated form if we want to
		assign attributes. This is done by having `AnyValue::set_attr` take a mutable reference to
		self. However, the only time this is useful is if we're talking about a named attribute---if
		we're assigning to an unnamed local, that means it'll just get thrown away immediately.

		As such, if it's an unnamed local, we still call the `set_attr`, in case it has a side effect,
		but we don't actually assign the `object` to anything. On the other hand, we have to box
		the `object` if it's not already a box.
		*/
		if let Ok(index) = usize::try_from(object_index) {
			let mut object = this.unnamed_locals[index];
			drop(this);
			object.set_attr(attr, value)?;
		} else {
			let index = !object_index as usize;
			let name = this.block.named_locals[index].as_any();
			let object = this.0.header_mut().get_unbound_attr_mut(name)?;
			object.set_attr(attr, value)?;
		}

		Ok(())
	}

	fn op_delattr(&self) -> Result<()>{
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;

		drop(this); // as `has_attr` may modify us.
		let value = object.del_attr(attr)?;
		self.as_mut()?.store_next_local(value.unwrap_or_default());

		Ok(())
	}

	fn op_callattr(&self) -> Result<()>{
		todo!("semantics for complicated callattr");
	}

	fn op_callattr_simple(&self) -> Result<()>{
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let amnt = this.next_count();
		let mut positional = Vec::with_capacity(amnt);

		for _ in 0..amnt {
			positional.push(this.next_local()?);
		}
		drop(this);
		let result = object.call_attr(attr, Args::new(&positional, &[]))?;
		self.as_mut()?.store_next_local(result);

		Ok(())
	}

	fn run_binary_op(&self, op: Intern) -> Result<()> {
		let mut this = self.as_mut()?;
		let lhs = this.next_local()?;
		let rhs = this.next_local()?;
		drop(this);

		let result = lhs.call_attr(op, Args::new(&[rhs], &[]))?;
		self.as_mut()?.store_next_local(result);

		Ok(())
	}
	
	fn op_add(&self) -> Result<()> {
		self.run_binary_op(Intern::op_add)
	}

	fn op_subtract(&self) -> Result<()> {
		self.run_binary_op(Intern::op_sub)
	}

	fn op_multuply(&self) -> Result<()> {
		self.run_binary_op(Intern::op_mul)
	}

	fn op_divide(&self) -> Result<()> {
		self.run_binary_op(Intern::op_div)
	}

	fn op_modulo(&self) -> Result<()> {
		self.run_binary_op(Intern::op_mod)
	}

	fn op_power(&self) -> Result<()> {
		self.run_binary_op(Intern::op_pow)
	}

	fn op_equal(&self) -> Result<()> {
		self.run_binary_op(Intern::op_eql)
	}

	fn op_notequal(&self) -> Result<()> {
		self.run_binary_op(Intern::op_neq)
	}

	fn op_lessthan(&self) -> Result<()> {
		self.run_binary_op(Intern::op_lth)
	}

	fn op_greaterthan(&self) -> Result<()> {
		self.run_binary_op(Intern::op_gth)
	}

	fn op_lessequal(&self) -> Result<()> {
		self.run_binary_op(Intern::op_leq)
	}

	fn op_greaterequal(&self) -> Result<()> {
		self.run_binary_op(Intern::op_geq)
	}

	fn op_compare(&self) -> Result<()> {
		self.run_binary_op(Intern::op_cmp)
	}

	fn op_index(&self) -> Result<()> {
		self.run_binary_op(Intern::op_index)
	}


	fn op_not(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let value = this.next_local()?;
		drop(this);

		let result = value.call_attr(Intern::op_not, Args::default())?;
		self.as_mut()?.store_next_local(result);

		Ok(())
	}

	fn op_negate(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let value = this.next_local()?;
		drop(this);

		let result = value.call_attr(Intern::op_neg, Args::default())?;
		self.as_mut()?.store_next_local(result);

		Ok(())
	}

	fn op_indexassign(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let ary = this.next_local()?;
		let index = this.next_local()?;
		let value = this.next_local()?;
		drop(this);

		ary.call_attr(Intern::op_index_assign, Args::new(&[ary, value], &[]))?;

		Ok(())
	}

	pub fn run(self) -> Result<AnyValue> {
		if !self.as_ref()?.flags().try_acquire_all_user(FLAG_CURRENTLY_RUNNING) {
			return Err("stackframe is currently running".to_string().into());
		}

		let result = self.run_();

		if !self.as_ref().expect("unable to remove running flag").flags().remove_user(FLAG_CURRENTLY_RUNNING) {
			panic!("unable to set it as not currently running??");
		}

		result
	}

	fn run_(self) -> Result<AnyValue> {
		loop {
			let op = if let Some(opcode) = self.as_mut()?.next_opcode() {
				opcode
			} else {
				break;
			};

			match op {
				Opcode::NoOp => self.op_noop(),
				Opcode::Debug => self.op_debug(),

				Opcode::Mov => self.op_mov()?,
				Opcode::Call => todo!("call"),
				Opcode::CallSimple => self.op_call_simple()?,
				Opcode::Return => return self.op_return(),

				Opcode::ConstLoad => self.op_constload()?,
				Opcode::CurrentFrame => self.op_currentframe()?,
				Opcode::GetAttr => self.op_getattr()?,
				Opcode::HasAttr => self.op_hasattr()?,
				Opcode::SetAttr => self.op_setattr()?,
				Opcode::DelAttr => self.op_delattr()?,
				Opcode::CallAttr => self.op_callattr()?,
				Opcode::CallAttrSimple => self.op_callattr_simple()?,

				Opcode::Add => self.op_add()?,
				Opcode::Subtract => self.op_subtract()?,
				Opcode::Multuply => self.op_multuply()?,
				Opcode::Divide => self.op_divide()?,
				Opcode::Modulo => self.op_modulo()?,
				Opcode::Power => self.op_power()?,

				Opcode::Not => self.op_not()?,
				Opcode::Negate => self.op_negate()?,
				Opcode::Equal => self.op_equal()?,
				Opcode::NotEqual => self.op_notequal()?,
				Opcode::LessThan => self.op_lessthan()?,
				Opcode::GreaterThan => self.op_greaterthan()?,
				Opcode::LessEqual => self.op_lessequal()?,
				Opcode::GreaterEqual => self.op_greaterequal()?,
				Opcode::Compare => self.op_compare()?,

				Opcode::Index => self.op_index()?,
				Opcode::IndexAssign => self.op_indexassign()?,
			}
		}

		Ok(AnyValue::default())
	}
}


quest_type_attrs! { for Gc<Frame>, parents [Kernel, Callable];
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}

