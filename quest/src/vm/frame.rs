use super::block::BlockInner;
use super::bytecode::Opcode;
use crate::value::base::{Base, Flags};
use crate::value::ty::{List, Text};
use crate::value::{AsAny, Gc, HasDefaultParent, Intern};
use crate::vm::bytecode::{COUNT_IS_NOT_ONE_BYTE_BUT_USIZE, MAX_ARGUMENTS_FOR_SIMPLE_CALL};
use crate::vm::{Args, Block};
use crate::{AnyValue, Error, Result};
use std::alloc::Layout;
use std::cell::UnsafeCell;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

quest_type! {
	#[derive(NamedType)]
	pub struct Frame(Inner);
}

impl Debug for Frame {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Frame({:p}:{:?})", self, self.0.data().block.loc)
	}
}

#[derive(Debug)]
pub struct Inner {
	block: Arc<BlockInner>,
	pos: usize,
	// note that both of these are actually from the same allocation;
	// `unnamed_locals` points to the base and `named_locals` is simply an offset.
	unnamed_locals: *mut AnyValue,
	named_locals: *mut Option<AnyValue>,
}

const FLAG_CURRENTLY_RUNNING: u32 = Flags::USER0;
const FLAG_IS_OBJECT: u32 = Flags::USER1;

fn locals_layout_for(num_of_unnamed_locals: usize, num_named_locals: usize) -> Layout {
	Layout::array::<Option<AnyValue>>(num_of_unnamed_locals + num_named_locals).unwrap()
}

impl Drop for Inner {
	fn drop(&mut self) {
		let layout =
			locals_layout_for(self.block.num_of_unnamed_locals, self.block.named_locals.len());

		unsafe {
			std::alloc::dealloc(self.unnamed_locals.cast::<u8>(), layout);
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct LocalTarget(isize);

impl Frame {
	pub fn new(gc_block: Gc<Block>, args: Args) -> Result<Gc<Frame>> {
		args.assert_no_keyword()?; // todo: assign keyword arguments
		let block = gc_block.as_ref()?.inner();

		if block.named_locals.len() < args.positional().len() {
			return Err(
				format!(
					"argc mismatch, expected at most {}, got {}",
					block.named_locals.len(),
					args.positional().len()
				)
				.into(),
			);
		}

		let mut builder = Base::<Inner>::builder();

		// XXX: If we swap these around, we get a significant speed slowdown. But what semantics
		// do we want? Do we want the outside stackframe to be first or last? and in any case,
		// this is setting the _block_ itself as the parent, which isn't what we want. how do we
		// want to register the outer block as the parent?
		builder.set_parents(List::from_slice(&[gc_block.as_any(), Gc::<Frame>::parent()]));

		unsafe {
			let unnamed_locals = crate::alloc_zeroed(locals_layout_for(
				block.num_of_unnamed_locals,
				block.named_locals.len(),
			))
			.as_ptr()
			.cast::<AnyValue>();

			let named_locals = unnamed_locals
				.add(block.num_of_unnamed_locals)
				.cast::<Option<AnyValue>>();

			// The scratch register defaults to null.
			unnamed_locals.write(AnyValue::default());

			// copy positional arguments over into the first few named local arguments.
			let mut start = named_locals;
			if let Some(this) = args.get_self() {
				named_locals.write(Some(this));
				start = named_locals.add(1);
			}
			start.copy_from_nonoverlapping(
				args.positional().as_ptr().cast::<Option<AnyValue>>(),
				args.positional().len(),
			);

			let mut data_ptr = builder.data_mut();
			std::ptr::addr_of_mut!((*data_ptr).unnamed_locals).write(unnamed_locals);
			std::ptr::addr_of_mut!((*data_ptr).named_locals).write(named_locals);
			std::ptr::addr_of_mut!((*data_ptr).block).write(block);
			// no need to initialize `pos` as it starts off as zero.

			Ok(Gc::from_inner(builder.finish()))
		}
	}

	pub fn with_stackframe<F: FnOnce(&mut Vec<Gc<Frame>>) -> T, T>(func: F) -> T {
		use std::cell::RefCell;
		thread_local! {
			static STACKFRAMES: RefCell<Vec<Gc<Frame>>> = RefCell::new(Vec::new());
		}

		STACKFRAMES.with(|sf| func(&mut sf.borrow_mut()))
	}

	fn is_object(&self) -> bool {
		self.0.header().flags().contains(FLAG_IS_OBJECT)
	}

	fn convert_to_object(&mut self) -> Result<()> {
		// If we're already an object, nothing else needed to be done.
		if !self.0.header().flags().try_acquire_all_user(FLAG_IS_OBJECT) {
			return Ok(());
		}

		let (header, data) = self.0.header_data_mut();
		// OPTIMIZE: we could use `with_capacity`, but we'd need to move it out of the builder.

		for i in 0..data.block.named_locals.len() {
			if let Some(value) = unsafe { *data.named_locals.add(i) } {
				header.set_attr(data.block.named_locals[i].as_any(), value)?;
			}
		}

		Ok(())
	}

	unsafe fn get_unnamed_local(&self, index: usize) -> AnyValue {
		debug_assert!(index <= self.block.num_of_unnamed_locals);

		unsafe {
			debug_assert!(
				self
					.unnamed_locals
					.add(index)
					.cast::<Option<AnyValue>>()
					.read()
					.is_some(),
				"reading from an unassigned unnamed local!"
			);

			*self.unnamed_locals.add(index)
		}
	}

	// this should also be unsafe
	fn get_local(&self, index: LocalTarget) -> Result<AnyValue> {
		let index = index.0;

		if 0 <= index {
			return Ok(unsafe { self.get_unnamed_local(index as usize) });
		}

		let index = !index as usize;

		if !self.is_object() {
			debug_assert!(index <= self.block.named_locals.len());

			// Since we could be trying to access a parent scope's variable, we won't return an error
			// in the false case.
			if let Some(value) = unsafe { *self.named_locals.add(index) } {
				return Ok(value);
			}
		}

		debug_assert!(index <= self.block.named_locals.len());
		let attr_name = unsafe { *self.block.named_locals.get_unchecked(index) };
		self
			.0
			.header()
			.get_unbound_attr(attr_name.as_any(), true)?
			.ok_or_else(|| format!("unknown attribute {:?}", attr_name).into())
	}

	fn set_local(&mut self, index: LocalTarget, value: AnyValue) -> Result<()> {
		let index = index.0;

		if 0 <= index {
			let index = index as usize;
			debug_assert!(index <= self.block.num_of_unnamed_locals);

			unsafe {
				self.unnamed_locals.add(index).write(value);
			}

			return Ok(());
		}

		let index = !index as usize;

		if !self.is_object() {
			debug_assert!(index <= self.block.named_locals.len());

			unsafe {
				self.named_locals.add(index).write(Some(value));
			}

			return Ok(());
		}

		debug_assert!(index <= self.block.named_locals.len());
		let attr_name = unsafe { *self.block.named_locals.get_unchecked(index) };
		self.0.header_mut().set_attr(attr_name.as_any(), value)
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
	fn is_done(&self) -> bool {
		self.pos >= self.block.code.len()
	}

	fn next_byte(&mut self) -> u8 {
		debug_assert!(self.pos < self.block.code.len());

		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		let byte = unsafe { *self.block.code.get_unchecked(self.pos) };

		trace!(target: "frame", byte=%format!("{:02x}", byte), sp=%self.pos, "read byte");

		self.pos += 1;
		trace!(target: "frame", ?byte, "read byte");
		byte
	}

	fn next_usize(&mut self) -> usize {
		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		let us = unsafe {
			self
				.block
				.code
				.as_ptr()
				.add(self.pos)
				.cast::<usize>()
				.read_unaligned()
		};

		self.pos += std::mem::size_of::<usize>();

		us
	}

	fn next_local(&mut self) -> Result<AnyValue> {
		let index = self.next_local_target();
		let value = self.get_local(index)?;

		trace!(target: "frame", ?index, ?value, "read local");

		Ok(value)
	}

	fn next_count(&mut self) -> usize {
		match self.next_byte() {
			COUNT_IS_NOT_ONE_BYTE_BUT_USIZE => self.next_usize(),
			byte if (byte as i8) < 0 => byte as i8 as isize as usize,
			byte => byte as usize,
		}
	}

	fn next_local_target(&mut self) -> LocalTarget {
		LocalTarget(self.next_count() as isize)
	}

	fn next_opcode(&mut self) -> Opcode {
		let byte = self.next_byte();

		let op = Opcode::from_u8(byte).unwrap_or_else(|| unreachable!("unknown opcode {:02x}", byte));
		trace!(target: "frame", ?op, "read opcode");
		op
	}
}

impl Gc<Frame> {
	fn op_mov(&self) -> Result<()> {
		let mut this = self.as_mut()?;

		let src = this.next_local()?;
		let dst = this.next_local_target();

		debug!(target: "frame", ?dst, ?src, "mov");
		this.set_local(dst, src);

		Ok(())
	}

	fn op_create_list(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let amnt = this.next_count();

		// TODO: use simple list builder when we make it
		let list = List::with_capacity(amnt);
		{
			let mut l = list.as_mut().unwrap();
			for i in 0..amnt {
				l.push(this.next_local()?);
			}
		}

		let dst = this.next_local_target();

		debug!(target: "frame", ?dst, ?list, "create_list");
		this.set_local(dst, list.as_any());

		Ok(())
	}

	fn op_call(&self) -> Result<()> {
		todo!()
	}

	fn op_call_simple(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let amnt = this.next_count();

		debug_assert!(amnt <= MAX_ARGUMENTS_FOR_SIMPLE_CALL);
		let mut positional = [MaybeUninit::<AnyValue>::uninit(); MAX_ARGUMENTS_FOR_SIMPLE_CALL];
		let ptr = positional.as_mut_ptr().cast::<AnyValue>();

		let args = unsafe {
			for i in 0..amnt {
				ptr.add(i).write(this.next_local()?);
			}
			std::slice::from_raw_parts(ptr, amnt)
		};

		let dst = this.next_local_target();

		drop(this);
		let result = object.call(Args::new(args, &[]))?;

		debug!(target: "frame", ?dst, ?object, ?args, ?result, "call_simple");
		self.as_mut()?.set_local(dst, result);

		Ok(())
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

		if let Some(block) = constant.downcast::<Gc<Block>>() {
			let block = block.as_ref()?.deep_clone()?;
			this.convert_to_object()?;

			block
				.as_ref()?
				.parents()?
				.as_list()
				.as_mut()?
				.unshift(self.clone().as_any());
		}

		let dst = this.next_local_target();

		debug!(target: "frame", ?dst, ?constant, "constload");
		this.set_local(dst, constant);

		Ok(())
	}

	fn op_stackframe(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let mut count = this.next_count() as isize;
		let dst = this.next_local_target();

		// todo: optimization for :0
		drop(this);
		let frame = Frame::with_stackframe(|frames| {
			if count < 0 {
				count += frames.len() as isize;
			}
			if count < 0 {
				panic!("todo: out of bounds error");
			}

			Result::<_>::Ok(
				frames
					.get(frames.len() - count as usize - 1)
					.expect("todo: out of bounds error")
					.clone(),
			)
		})?;
		frame.as_mut()?.convert_to_object()?;

		debug!(target: "frame", ?dst, ?frame, "stackframe");
		self.as_mut()?.set_local(dst, frame.as_any());

		Ok(())
	}

	fn op_get_attr(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let dst = this.next_local_target();

		drop(this);
		let value = object
			.get_attr(attr)?
			.ok_or_else(|| format!("unknown attr {:?} for {:?}", attr, object))?;

		debug!(target: "frame", ?dst, ?object, ?attr, ?value, "get_attr");
		self.as_mut()?.set_local(dst, value);

		Ok(())
	}

	fn op_get_unbound_attr(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let dst = this.next_local_target();

		drop(this); // as `get_attr` may modify us.
		let value = object
			.get_unbound_attr(attr)?
			.ok_or_else(|| format!("unknown attr {:?} for {:?}", attr, object))?;

		debug!(target: "frame", ?dst, ?object, ?attr, ?value, "get_unbound_attr");
		self.as_mut()?.set_local(dst, value);

		Ok(())
	}

	fn op_has_attr(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let dst = this.next_local_target();

		drop(this); // as `has_attr` may modify us.
		let hasit = object.has_attr(attr)?;

		debug!(target: "frame", ?dst, ?object, ?attr, ?hasit, "has_attr");
		self.as_mut()?.set_local(dst, hasit.as_any());

		Ok(())
	}

	fn op_set_attr(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object_index = this.next_count() as isize;
		let attr = this.next_local()?;
		let value = this.next_local()?;
		let dst = this.next_local_target();

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
		let object = if 0 <= object_index {
			let mut object = unsafe { this.get_unnamed_local(object_index as usize) };
			object.set_attr(attr, value)?;
			object
		} else {
			let index = !object_index as usize;
			let name = this.block.named_locals[index].as_any();
			let object = this.0.header_mut().get_unbound_attr_mut(name)?;
			object.set_attr(attr, value)?;
			*object
		};

		debug!(target: "frame", ?dst, ?object, ?attr, ?value, "set_attr");
		this.set_local(dst, value);

		Ok(())
	}

	fn op_del_attr(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let dst = this.next_local_target();

		drop(this); // as `has_attr` may modify us.
		let value = object.del_attr(attr)?;

		debug!(target: "frame", ?dst, ?object, ?attr, ?value, "del_attr");
		self.as_mut()?.set_local(dst, value.unwrap_or_default());

		Ok(())
	}

	fn op_call_attr(&self) -> Result<()> {
		todo!("semantics for complicated callattr");
	}

	fn op_call_attr_simple(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let object = this.next_local()?;
		let attr = this.next_local()?;
		let amnt = this.next_count();

		debug_assert!(amnt <= MAX_ARGUMENTS_FOR_SIMPLE_CALL);
		let mut positional = [MaybeUninit::<AnyValue>::uninit(); MAX_ARGUMENTS_FOR_SIMPLE_CALL];
		let ptr = positional.as_mut_ptr().cast::<AnyValue>();

		let args = unsafe {
			for i in 0..amnt {
				ptr.add(i).write(this.next_local()?);
			}
			std::slice::from_raw_parts(ptr, amnt)
		};

		let dst = this.next_local_target();
		drop(this);
		let result = object.call_attr(attr, Args::new(args, &[]))?;

		debug!(target: "frame", ?dst, ?object, ?attr, ?args, ?result, "call_attr_simple");
		self.as_mut()?.set_local(dst, result);

		Ok(())
	}

	fn run_binary_op(&self, op: Intern) -> Result<()> {
		let mut this = self.as_mut()?;
		let lhs = this.next_local()?;
		let rhs = this.next_local()?;
		let dst = this.next_local_target();

		drop(this);
		let result = lhs.call_attr(op, Args::new(&[rhs], &[]))?;

		debug!(target: "frame", ?dst, ?op, ?lhs, ?rhs, ?result, "binary_op");
		self.as_mut()?.set_local(dst, result);

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
		let mut this = self.as_mut()?;
		let source = this.next_local()?;
		let argc = this.next_count();
		// todo: optimize me not to use a `Vec`.
		let mut args = Vec::with_capacity(argc + 1);
		for i in 0..argc {
			args.push(this.next_local()?);
		}
		let dst = this.next_local_target();

		drop(this);
		let result = source.call_attr(Intern::op_index, Args::new(&args, &[]))?;

		debug!(target: "frame", ?dst, ?source, ?args, ?result, "index");
		self.as_mut()?.set_local(dst, result);

		Ok(())
	}

	fn op_not(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let value = this.next_local()?;
		let dst = this.next_local_target();

		drop(this);
		let result = value.call_attr(Intern::op_not, Args::default())?;

		debug!(target: "frame", ?dst, ?value, ?result, "not");
		self.as_mut()?.set_local(dst, result);

		Ok(())
	}

	fn op_negate(&self) -> Result<()> {
		let mut this = self.as_mut()?;
		let value = this.next_local()?;
		let dst = this.next_local_target();

		drop(this);
		let result = value.call_attr(Intern::op_neg, Args::default())?;

		debug!(target: "frame", ?dst, ?value, ?result, "neg");
		self.as_mut()?.set_local(dst, result);

		Ok(())
	}

	fn op_indexassign(&self) -> Result<()> {
		// todo: optimize me not to use a `Vec`.
		let mut this = self.as_mut()?;
		let source = this.next_local()?;
		let argc = this.next_count();

		let mut args = Vec::with_capacity(argc + 1);
		for i in 0..argc {
			args.push(this.next_local()?);
		}
		let value = this.next_local()?;
		args.push(value);

		let dst = this.next_local_target();

		drop(this);
		let result = source.call_attr(Intern::op_index_assign, Args::new(&args, &[]))?;

		debug!(target: "frame", ?dst, ?source, ?args, ?result, "index_assign");
		self.as_mut()?.set_local(dst, result);

		Ok(())
	}

	pub fn run(self) -> Result<AnyValue> {
		if !self
			.as_ref()?
			.flags()
			.try_acquire_all_user(FLAG_CURRENTLY_RUNNING)
		{
			return Err("stackframe is currently running".to_string().into());
		}

		Frame::with_stackframe(|sfs| sfs.push(self));

		let result = self.run_inner();

		if !self
			.as_ref()
			.expect("unable to remove running flag")
			.flags()
			.remove_user(FLAG_CURRENTLY_RUNNING)
		{
			unreachable!("unable to set it as not currently running??");
		}

		Frame::with_stackframe(|sfs| {
			if cfg!(debug_assertions) {
				debug_assert!(sfs.pop().unwrap().ptr_eq(self));
			} else {
				sfs.pop();
			}
		});

		result?;

		// read the implicit return value
		self.as_mut().map(|this| unsafe { *this.unnamed_locals })
	}

	fn next_op(&mut self) -> Result<Option<Opcode>> {
		let mut m = self.as_mut()?;
		if m.is_done() {
			Ok(None)
		} else {
			Ok(Some(m.next_opcode()))
		}
	}

	fn run_inner(mut self) -> Result<()> {
		while let Some(op) = self.next_op()? {
			match op {
				Opcode::CreateList => self.op_create_list(),
				Opcode::Mov => self.op_mov(),
				Opcode::Call => self.op_call(),
				Opcode::CallSimple => self.op_call_simple(),

				Opcode::ConstLoad => self.op_constload(),
				Opcode::Stackframe => self.op_stackframe(),
				Opcode::GetAttr => self.op_get_attr(),
				Opcode::GetUnboundAttr => self.op_get_unbound_attr(),
				Opcode::HasAttr => self.op_has_attr(),
				Opcode::SetAttr => self.op_set_attr(),
				Opcode::DelAttr => self.op_del_attr(),
				Opcode::CallAttr => self.op_call_attr(),
				Opcode::CallAttrSimple => self.op_call_attr_simple(),

				Opcode::Add => self.op_add(),
				Opcode::Subtract => self.op_subtract(),
				Opcode::Multuply => self.op_multuply(),
				Opcode::Divide => self.op_divide(),
				Opcode::Modulo => self.op_modulo(),
				Opcode::Power => self.op_power(),

				Opcode::Not => self.op_not(),
				Opcode::Negate => self.op_negate(),
				Opcode::Equal => self.op_equal(),
				Opcode::NotEqual => self.op_notequal(),
				Opcode::LessThan => self.op_lessthan(),
				Opcode::GreaterThan => self.op_greaterthan(),
				Opcode::LessEqual => self.op_lessequal(),
				Opcode::GreaterEqual => self.op_greaterequal(),
				Opcode::Compare => self.op_compare(),

				Opcode::Index => self.op_index(),
				Opcode::IndexAssign => self.op_indexassign(),
			}?;
		}

		Ok(())
	}
}

pub mod funcs {
	use super::*;

	pub fn resume(frame: Gc<Frame>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;

		frame.run()
	}

	pub fn restart(frame: Gc<Frame>, args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_arguments()?;
		frame.as_mut()?.pos = 0;

		frame.run()
	}
}

quest_type_attrs! { for Gc<Frame>, parents [Kernel, Callable];
	resume => meth funcs::resume,
	restart => meth funcs::restart,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fibonacci() {
		let fib = {
			let mut builder = crate::vm::block::Builder::new(Default::default(), None);

			let n = builder.named_local("n");
			let fib = builder.named_local("fib");
			let one = builder.unnamed_local();
			let tmp = builder.unnamed_local();
			let tmp2 = builder.unnamed_local();
			let tmp3 = builder.unnamed_local();
			let ret = builder.unnamed_local();

			builder.constant(1.as_any(), one);
			builder.less_equal(n, one, tmp);
			builder.constant("then".as_any(), tmp2);
			builder.constant("return".as_any(), ret);
			builder.get_attr(n, ret, tmp3);
			builder.call_attr_simple(tmp, tmp2, &[tmp3], tmp);
			builder.subtract(n, one, n);
			builder.call_simple(fib, &[n], tmp);
			builder.subtract(n, one, n);
			builder.call_simple(fib, &[n], tmp2);
			builder.add(tmp, tmp2, tmp);
			builder.call_attr_simple(tmp, ret, &[], tmp);

			builder.build()
		};

		fib.as_mut()
			.unwrap()
			.set_attr("fib".as_any(), fib.as_any())
			.unwrap();

		let result = fib.run(Args::new(&[15.as_any()], &[])).unwrap();

		assert_eq!(result.downcast::<crate::value::ty::Integer>(), Some(610));
	}
}
