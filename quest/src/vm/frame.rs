//! Types associated with the [`Frame`] type.

use crate::value::base::{Base, Flags};
use crate::value::ty::{List, Text};
use crate::value::{Gc, HasDefaultParent, Intern, ToValue};
use crate::vm::block::BlockInner;
use crate::vm::{Args, Block, Opcode, COUNT_IS_NOT_ONE_BYTE_BUT_USIZE, NUM_ARGUMENT_REGISTERS};
use crate::{ErrorKind, Result, Value};
use std::alloc::Layout;
use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

quest_type! {
	/// A Stackframe within quest.
	#[derive(NamedType)]
	pub struct Frame(Inner);
}

impl Debug for Frame {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Frame({:p}:{:?})", self, self.0.data().inner_block.location)
	}
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Inner {
	block: Gc<Block>,
	inner_block: Arc<BlockInner>,
	pos: usize,

	// note that both of these are actually from the same allocation;
	// `unnamed_locals` points to the base and `named_locals` is simply an offset.
	unnamed_locals: *mut Value,
	named_locals: *mut Option<Value>,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Drop for Inner {
	fn drop(&mut self) {
		let layout = locals_layout_for(
			self.inner_block.num_of_unnamed_locals,
			self.inner_block.named_locals.len(),
		);

		unsafe {
			std::alloc::dealloc(self.unnamed_locals.cast::<u8>(), layout);
		}
	}
}

const FLAG_CURRENTLY_RUNNING: u32 = Flags::USER0;
const FLAG_IS_OBJECT: u32 = Flags::USER1;

fn locals_layout_for(num_of_unnamed_locals: usize, num_named_locals: usize) -> Layout {
	Layout::array::<Option<Value>>(num_of_unnamed_locals + num_named_locals).unwrap()
}

#[derive(Debug, Clone, Copy)]
struct LocalTarget(isize);

impl Frame {
	/// Creates a new [`Frame`] from the given `block` and passed `args`.
	pub fn new(block: Gc<Block>, args: Args) -> Result<Gc<Self>> {
		args.assert_no_keyword()?; // todo: assign keyword arguments
		let inner_block = block.as_ref()?.inner();

		if inner_block.named_locals.len() < args.positional().len() {
			// TODO: Arity
			return Err(
				ErrorKind::PositionalArgumentMismatch {
					given: args.positional().len(),
					expected: inner_block.named_locals.len(),
				}
				.into(),
			);
		}

		let mut builder = Base::<Inner>::builder();

		// XXX: If we swap these around, we get a significant speed slowdown. But what semantics
		// do we want? Do we want the outside stackframe to be first or last? and in any case,
		// this is setting the _block_ itself as the parent, which isn't what we want. how do we
		// want to register the outer block as the parent?
		// ^^ update: having them like `[block, parent]` means that attribute lookups such as `dbg`
		// are first found on frame, which is not good.
		// update 2: we removed the following, as we simply set the `Gc::parent()` when we convert to
		// an actual object.
		// 	builder.set_parents(List::from_slice(&[Gc::<Self>::parent(), block.to_value()]));
		builder.set_parents(block.to_value());

		unsafe {
			let unnamed_locals = crate::alloc_zeroed::<Value>(locals_layout_for(
				inner_block.num_of_unnamed_locals,
				inner_block.named_locals.len(),
			))
			.as_ptr();

			let named_locals =
				unnamed_locals.add(inner_block.num_of_unnamed_locals).cast::<Option<Value>>();

			// The scratch register defaults to null.
			unnamed_locals.write(Value::default());

			// copy positional arguments over into the first few named local arguments.
			let mut start = named_locals;
			start.add(0).write(Some(block.to_value()));
			start.add(1).write(Some(args.into_value()));
			start = start.add(2);

			if let Some(this) = args.this() {
				start.write(Some(this));
				start = start.add(1);
			}

			start.copy_from_nonoverlapping(
				args.positional().as_ptr().cast::<Option<Value>>(),
				args.positional().len(),
			);

			let data_ptr = builder.data_mut();

			std::ptr::addr_of_mut!((*data_ptr).unnamed_locals).write(unnamed_locals);
			std::ptr::addr_of_mut!((*data_ptr).named_locals).write(named_locals);
			std::ptr::addr_of_mut!((*data_ptr).inner_block).write(inner_block);
			std::ptr::addr_of_mut!((*data_ptr).block).write(block);
			// no need to initialize `pos` as it starts off as zero.

			Ok(Gc::from_inner(builder.finish()))
		}
	}

	/// Fetches the block associated with this stackframe.
	pub fn block(&self) -> Gc<Block> {
		self.block
	}

	pub(crate) fn is_object(&self) -> bool {
		self.0.header().flags().contains(FLAG_IS_OBJECT)
	}

	pub(super) fn convert_to_object(&mut self) -> Result<()> {
		// If we're already an object, nothing else needed to be done.
		if !self.0.header().flags().try_acquire_all_user(FLAG_IS_OBJECT) {
			return Ok(());
		}

		let block = self.0.data().block.to_value();
		let (header, data) = self.0.header_data_mut();

		// Once we start referencing the frame as an object, we no longer can longer use the "block is
		// our only parent" optimization.
		header.parents_mut().set(List::from_slice(&[Gc::<Self>::parent(), block]));
		// OPTIMIZE: we could use `with_capacity`, but we'd need to move it out of the builder.

		for i in 0..data.inner_block.named_locals.len() {
			if let Some(value) = unsafe { *data.named_locals.add(i) } {
				header.set_attr(data.inner_block.named_locals[i].to_value(), value)?;
			}
		}

		Ok(())
	}

	unsafe fn get_unnamed_local(&self, index: usize) -> Value {
		debug_assert!(
			index <= self.inner_block.num_of_unnamed_locals,
			"{:?} > {:?}",
			index,
			self.inner_block.num_of_unnamed_locals
		);
		debug_assert!(
			self.unnamed_locals.add(index).cast::<Option<Value>>().read().is_some(),
			"reading from an unassigned unnamed local!"
		);

		*self.unnamed_locals.add(index)
	}

	// this should also be unsafe (update: should it be??)
	fn get_local(&self, index: LocalTarget) -> Result<Value> {
		let index = index.0;

		if 0 <= index {
			return Ok(unsafe { self.get_unnamed_local(index as usize) });
		}

		let index = !index as usize;
		debug_assert!(index <= self.inner_block.named_locals.len());

		if !self.is_object() {
			// Since we could be trying to access a parent scope's variable, we won't return an error
			// in the false case.
			if let Some(value) = unsafe { *self.named_locals.add(index) } {
				return Ok(value);
			}
		}

		self.get_object_local(index)
	}

	// The vast majority of the time, we're looking for unnamed or named locals, not through parent
	// attributes.
	#[inline(never)]
	fn get_object_local(&self, index: usize) -> Result<Value> {
		let attr_name = unsafe { *self.inner_block.named_locals.get_unchecked(index) };

		if let Some(attr) =
			self.0.header().get_unbound_attr_checked(attr_name.to_value(), &mut Vec::new())?
		{
			return Ok(attr);
		} else if !self.is_object() {
			if let Some(attr) = Gc::<Self>::parent().get_unbound_attr(attr_name.to_value())? {
				return Ok(attr);
			}
		}

		Err(
			ErrorKind::UnknownAttribute {
				object: unsafe { crate::value::Gc::new(self.into()) }.to_value(),
				attribute: attr_name.to_value(),
			}
			.into(),
		)
	}

	fn set_local(&mut self, index: LocalTarget, value: Value) -> Result<()> {
		let index = index.0;

		if 0 <= index {
			let index = index as usize;
			debug_assert!(index <= self.inner_block.num_of_unnamed_locals);

			unsafe {
				self.unnamed_locals.add(index).write(value);
			}

			return Ok(());
		}

		let index = !index as usize;
		debug_assert!(index <= self.inner_block.named_locals.len());

		if !self.is_object() {
			unsafe {
				self.named_locals.add(index).write(Some(value));
			}

			return Ok(());
		}

		self.set_object_local(index, value)
	}

	#[inline(never)]
	fn set_object_local(&mut self, index: usize, value: Value) -> Result<()> {
		let attr_name = unsafe { *self.inner_block.named_locals.get_unchecked(index) };
		self.0.header_mut().set_attr(attr_name.to_value(), value)
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
		self.pos >= self.inner_block.code.len()
	}

	fn next_byte(&mut self) -> u8 {
		debug_assert!(!self.is_done());

		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		let byte = unsafe { *self.inner_block.code.get_unchecked(self.pos) };

		trace!(target: "frame", byte=%format!("{byte:02x}"), sp=%self.pos, "read byte");

		self.pos += 1;
		byte
	}

	#[cold]
	fn next_usize(&mut self) -> usize {
		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		#[allow(clippy::cast_ptr_alignment)]
		let us = unsafe { self.inner_block.code.as_ptr().add(self.pos).cast::<usize>().read_unaligned() };

		self.pos += std::mem::size_of::<usize>();

		us
	}

	fn next_local(&mut self) -> Result<Value> {
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

		debug_assert!(Opcode::verify_is_valid(byte), "read invalid opcode? {:?}", byte);

		let op = unsafe { std::mem::transmute::<u8, Opcode>(byte) };

		trace!(target: "frame", ?op, "read opcode");
		op
	}

	fn next_op(&mut self) -> Result<Option<Opcode>> {
		if self.is_done() {
			Ok(None)
		} else {
			Ok(Some(self.next_opcode()))
		}
	}

	// safety: index has to be in bounds
	unsafe fn get_constant(&mut self, index: usize, dst: LocalTarget) -> Result<Value> {
		debug_assert!(index <= self.inner_block.constants.len());

		let constant = *self.inner_block.constants.get_unchecked(index);
		if let Some(block) = constant.downcast::<Gc<Block>>() {
			self.constant_as_block(block, dst)
		} else {
			Ok(constant)
		}
	}

	#[inline(never)]
	fn constant_as_block(&mut self, block: Gc<Block>, dst: LocalTarget) -> Result<Value> {
		self.convert_to_object()?;

		let parent = unsafe { crate::value::Gc::new(self.into()) };
		let block = block.as_ref()?.deep_clone_from(parent)?;

		// TODO: maybe pass name to `deep_clone_from` too?
		if dst.0 < 0 {
			let index = !dst.0 as usize;
			debug_assert!(index <= self.inner_block.named_locals.len());
			let name = unsafe { *self.inner_block.named_locals.get_unchecked(index) };
			block.as_mut().unwrap().set_name(name)?;
		}

		Ok(block.to_value())
	}
}

/// The maximum stackframe length.
pub const MAX_STACKFRAME_LEN: usize = if cfg!(debug_assertions) { 50 } else { 10_000 };

thread_local! {
	static STACKFRAMES: RefCell<Vec<Gc<Frame>>> = RefCell::new(
		Vec::with_capacity(MAX_STACKFRAME_LEN)
	);
}

/// Provides access to the stackframe.
pub fn with_stackframes<F: FnOnce(&[Gc<Frame>]) -> T, T>(func: F) -> T {
	STACKFRAMES.with(|sf| func(&sf.borrow()))
}

impl Gc<Frame> {
	/// Enters the given `frame`, executes `func`, then returns the result of `func`.
	pub fn enter_stackframe<F: FnOnce() -> Result<T>, T>(self, func: F) -> Result<T> {
		STACKFRAMES.with(|stackframes| {
			let mut sf = stackframes.borrow_mut();

			if MAX_STACKFRAME_LEN < sf.len() {
				drop(sf); // so we dont have a mutable borrow
				return Err(ErrorKind::StackOverflow.into());
			}

			sf.push(self);
			drop(sf); // so we can call `func`.

			let result = func();

			let popped_frame = stackframes.borrow_mut().pop();
			debug_assert!(popped_frame.unwrap().ptr_eq(self));

			result
		})
	}

	/// Executes the stackframe, returning an error if it's currently running.
	#[instrument(target="frame",
		level="debug",
		name="call frame",
		skip(self),
		fields(src=?self.as_ref()?.inner_block.location))
	]
	pub fn run(self) -> Result<Value> {
		if !self.as_ref()?.flags().try_acquire_all_user(FLAG_CURRENTLY_RUNNING) {
			return Err(ErrorKind::StackframeIsCurrentlyRunning(self).into());
		}

		let result = self.enter_stackframe(|| self.run_inner());

		if !self
			.as_ref()
			.expect("unable to remove running flag")
			.flags()
			.remove_user(FLAG_CURRENTLY_RUNNING)
		{
			unreachable!("unable to set it as not currently running??");
		}

		if let Err(err) = result {
			if let ErrorKind::Return { value, from_frame } = err.kind() {
				if from_frame.map_or(true, |ff| ff.is_identical(self.to_value())) {
					return Ok(*value);
				}
			}

			Err(err)
		} else {
			self.as_mut().map(|this| unsafe { *this.unnamed_locals })
		}
	}

	fn run_inner(self) -> Result<()> {
		let mut args = [MaybeUninit::<Value>::uninit(); NUM_ARGUMENT_REGISTERS];
		let mut this = self.as_mut()?;
		let mut variable_args_count = MaybeUninit::uninit();

		macro_rules! without_this {
			($($code:tt)*) => {{
				drop(this);
				let x = { $($code)* };
				this = self.as_mut()?;
				x
			}};
		}

		macro_rules! args_slice {
			(start=$start:expr) => {
				args_slice!(start = $start, len = variable_args_count.assume_init())
			};
			(start=$start:expr, len=$len:expr) => {
				Args::new(
					std::slice::from_raw_parts(args.as_ptr().cast::<Value>().add($start), $len),
					&[],
				)
			};
		}

		while let Some(op) = this.next_op()? {
			if cfg!(debug_assertions) {
				for position in args.iter_mut().take(NUM_ARGUMENT_REGISTERS) {
					*position = MaybeUninit::uninit();
				}

				variable_args_count = MaybeUninit::uninit();
			}

			let dst = this.next_local_target();

			{
				let arity = op.fixed_arity();
				let is_variable_simple = op.is_variable_simple();
				debug_assert!(arity <= NUM_ARGUMENT_REGISTERS);
				let mut ptr = args.as_mut_ptr().cast::<Value>();

				for _ in 0..arity {
					let local = this.next_local()?;

					unsafe {
						ptr.write(local);
						ptr = ptr.add(1);
					}
				}

				if is_variable_simple {
					let count = this.next_byte() as usize;
					variable_args_count.write(count);

					// all things with `is_variable` are <= NUM_ARGUMENT_REGISTERS.
					debug_assert_ne!(count, COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize);
					debug_assert!((count as u8 as i8) >= 0);
					debug_assert!(count <= NUM_ARGUMENT_REGISTERS);
					debug_assert!(arity + count <= NUM_ARGUMENT_REGISTERS);

					for _ in 0..count {
						let local = this.next_local()?;

						unsafe {
							ptr.write(local);
							ptr = ptr.add(1);
						}
					}
				}
			}

			let result = match op {
				// todo: create list short, do a bitwise copy over to the pointer.
				Opcode::CreateList | Opcode::CreateListShort => {
					let amnt = this.next_count();

					// TODO: use simple list builder when we make it
					let list = List::with_capacity(amnt);
					{
						let mut l = list.as_mut().unwrap();
						for _ in 0..amnt {
							l.push(this.next_local()?);
						}
					}

					list.to_value()
				}

				Opcode::Mov => unsafe { args[0].assume_init() },
				Opcode::Call => todo!(), //self.op_call(),
				Opcode::CallSimple => unsafe {
					without_this! {
						args[0].assume_init().call(args_slice!(start=1))?
					}
				},

				Opcode::ConstLoad => unsafe {
					let idx = this.next_count();
					this.get_constant(idx, dst)?
				},
				Opcode::Stackframe => {
					let mut count = this.next_count() as isize;

					// todo: optimization for :0
					let frame = with_stackframes(|frames| {
						if count < 0 {
							count += frames.len() as isize;

							if count < 0 {
								return Err("todo: out of bounds error".to_string().into());
							}
						}

						Result::<_>::Ok(
							*frames
								.get(frames.len() - count as usize - 1)
								.expect("todo: out of bounds error"),
						)
					})?;
					without_this! {
						frame.as_mut()?.convert_to_object()?;
					}
					frame.to_value()
				}

				Opcode::GetAttr => unsafe {
					without_this! {
						args[0].assume_init().try_get_attr(args[1].assume_init())?
					}
				},
				Opcode::GetUnboundAttr => unsafe {
					without_this! {
						args[0].assume_init().try_get_unbound_attr(args[1].assume_init())?
					}
				},
				Opcode::HasAttr => unsafe {
					without_this! {
						args[0].assume_init().has_attr(args[1].assume_init())?.to_value()
					}
				},
				Opcode::SetAttr => {
					let attr = unsafe { args[0].assume_init() };
					let value = unsafe { args[1].assume_init() };
					let object_index = this.next_count() as isize;

					/*
					Because you can assign indices onto any object, we need to be able to dynamically convert
					immediates (eg integers, floats, booleans, etc) into a heap-allocated form if we want to
					assign attributes. This is done by having `Value::set_attr` take a mutable reference to
					self. However, the only time this is useful is if we're talking about a named attribute---if
					we're assigning to an unnamed local, that means it'll just get thrown away immediately.

					As such, if it's an unnamed local, we still call the `set_attr`, in case it has a side effect,
					but we don't actually assign the `object` to anything. On the other hand, we have to box
					the `object` if it's not already a box.
					*/
					if 0 <= object_index {
						let mut object = unsafe { this.get_unnamed_local(object_index as usize) };

						if self.to_value().is_identical(object) {
							this.convert_to_object()?;
							this.set_attr(attr, value)?;
						} else {
							object.set_attr(attr, value)?;
						}
					} else {
						let index = !object_index as usize;
						let name = this.inner_block.named_locals[index].to_value();
						let object = this.0.header_mut().get_unbound_attr_mut(name)?;

						if self.to_value().is_identical(*object) {
							this.convert_to_object()?;
							this.set_attr(attr, value)?;
						} else {
							object.set_attr(attr, value)?;
						}
					}

					value
				}

				Opcode::DelAttr => unsafe {
					without_this! {
						args[0].assume_init().del_attr(args[1].assume_init())?.unwrap_or_default()
					}
				},
				Opcode::CallAttr => todo!(),
				Opcode::CallAttrSimple => unsafe {
					without_this! {
						args[0]
							.assume_init()
							.call_attr(args[1].assume_init(), args_slice!(start=2))?
					}
				},

				Opcode::Add => unsafe {
					without_this! {
						args[0].assume_init()
							.call_attr(Intern::op_add, args_slice!(start=1, len=1))?
					}
				},
				Opcode::Subtract => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_sub, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Multiply => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_mul, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Divide => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_div, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Modulo => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_mod, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Power => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_pow, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Not => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_not, Args::default())?
					}
				},
				Opcode::Negate => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_neg, Args::default())?
					}
				},
				Opcode::Equal => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_eql, args_slice!(start=1,len=1))?
					}
				},
				Opcode::NotEqual => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_neq, args_slice!(start=1,len=1))?
					}
				},
				Opcode::LessThan => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_lth, args_slice!(start=1,len=1))?
					}
				},
				Opcode::GreaterThan => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_gth, args_slice!(start=1,len=1))?
					}
				},
				Opcode::LessEqual => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_leq, args_slice!(start=1,len=1))?
					}
				},
				Opcode::GreaterEqual => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_geq, args_slice!(start=1,len=1))?
					}
				},
				Opcode::Compare => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_cmp, args_slice!(start=1,len=1))?
					}
				},

				Opcode::Index => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_index, args_slice!(start=1))?
					}
				},

				Opcode::IndexAssign => unsafe {
					without_this! {
						args[0].assume_init().call_attr(Intern::op_index_assign, args_slice!(start=1))?
					}
				},
			};

			debug!(target: "frame", ?dst, ?args, ?op, "ran opcode");
			this.set_local(dst, result)?;
		}

		Ok(())
	}
}

/// Quest functions defined for [`Block`].
pub mod funcs {
	use super::*;

	/// Resumes the `frame`. Note that if it's already running, an error will be returned.
	pub fn resume(frame: Gc<Frame>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		frame.run()
	}

	/// Restarts `frame` from the beginning. Note that if it's already running, an error will be
	/// returned.
	pub fn restart(frame: Gc<Frame>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;
		frame.as_mut()?.pos = 0;

		frame.run()
	}

	/// Returns a debug representation of `frame`.
	pub fn dbg(frame: Gc<Frame>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		// TODO: maybe cache this in the future?
		let mut builder = Text::simple_builder();
		builder.push_str("<Frame:");
		builder.push_str(&format!("{:p}", frame.to_value().bits() as *const u8));
		builder.push(':');
		builder.push_str(&frame.as_ref()?.inner_block.location.to_string());
		builder.push('>');

		Ok(builder.finish().to_value())
	}
}

quest_type_attrs! { for Gc<Frame>, parents [Kernel, Callable];
	resume => meth funcs::resume,
	restart => meth funcs::restart,
	dbg => meth funcs::dbg,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fibonacci() {
		let fib = {
			let mut builder = crate::vm::block::Builder::default();

			let n = builder.named_local("n");
			let fib = builder.named_local("fib");
			let one = builder.unnamed_local();
			let tmp = builder.unnamed_local();
			let tmp2 = builder.unnamed_local();
			let tmp3 = builder.unnamed_local();
			let ret = builder.unnamed_local();

			builder.constant(1.to_value(), one);
			builder.less_equal(n, one, tmp);
			builder.constant("then".to_value(), tmp2);
			builder.constant("return".to_value(), ret);
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

		fib.as_mut().unwrap().set_attr("fib".to_value(), fib.to_value()).unwrap();

		let result = fib.run(Args::new(&[15.to_value()], &[])).unwrap();

		assert_eq!(result.downcast::<crate::value::ty::Integer>(), Some(610));
	}
}
