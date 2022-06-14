//! Types associated with the [`Frame`] type.

use crate::value::base::{Base, Flags};
use crate::value::gc::Allocated;
use crate::value::ty::{List, Text};
use crate::value::{
	Attributed, AttributedMut, Callable, Gc, HasDefaultParent, HasParents, Intern, ToValue,
};
use crate::vm::block::BlockInner;
use crate::vm::{Args, Block, Opcode, COUNT_IS_NOT_ONE_BYTE_BUT_USIZE, NUM_ARGUMENT_REGISTERS};
use crate::{Error, ErrorKind, Result, Value};
use std::alloc::Layout;
use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::num::NonZeroUsize;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

quest_type! {
	/// A Stackframe within quest.
	// TODO: when garbage collecting, check `block`'s bytecode for immediates.
	#[derive(NamedType)]
	pub struct Frame(Inner);
}

impl Debug for Frame {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Frame({:p}:{:?})", self, self.0.data().inner_block.location)
	}
}

#[doc(hidden)]
pub struct Inner {
	block: Gc<Block>,
	// NOTE: We're guaranteed to have a well-defined `inner_block` because the only way to construct
	// it is through the builder, which guarantees creation of well-defined bytecode
	inner_block: Arc<BlockInner>,
	pos: usize,

	// note that both of these are actually from the same allocation;
	// `unnamed_locals` points to the base and `named_locals` is simply an offset.
	unnamed_locals: *mut Option<Value>,
	named_locals: *mut Option<Value>,
}

// TODO: verify send and sync for this.
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Drop for Inner {
	fn drop(&mut self) {
		let layout = locals_layout_for(
			self.inner_block.num_of_unnamed_locals,
			self.inner_block.named_locals.len(),
		);

		// SAFETY:
		// `self.unnamed_locals` was allocated by `crate::alloc`, and the layout is the same.
		unsafe {
			std::alloc::dealloc(self.unnamed_locals.cast::<u8>(), layout);
		}
	}
}

const FLAG_CURRENTLY_RUNNING: u32 = Flags::USER0;
const FLAG_IS_OBJECT: u32 = Flags::USER1;

// SAFETY: `num_of_unnamed_locals` should be nonzero
fn locals_layout_for(num_of_unnamed_locals: NonZeroUsize, num_named_locals: usize) -> Layout {
	// SAFETY: we know `num_of_unnamed_locals` is nonzero, as it's a `NonZeroUsize`.
	unsafe {
		Layout::array::<Option<Value>>(num_of_unnamed_locals.get() + num_named_locals)
			.unwrap_unchecked()
	}
}

#[derive(Debug, Clone, Copy)]
enum LocalTarget {
	Unnamed(usize),
	Named(usize),
}

impl Frame {
	/// Creates a new [`Frame`] from the given `block` and passed `args`.
	pub fn new(block: Gc<Block>, args: Args) -> Result<Gc<Self>> {
		args.assert_no_keyword().expect("todo: assign keyword arguments");

		let inner_block = block.as_ref()?.inner();
		if inner_block.arity != args.len() {
			return Err(
				ErrorKind::PositionalArgumentMismatch {
					given: args.len(),
					expected: inner_block.arity,
				}
				.into(),
			);
		}

		// SAFETY: `locals_layout_for` is guaranteed to have a positive size, because of
		// the scratch register.
		let unnamed_locals = unsafe {
			let layout =
				locals_layout_for(inner_block.num_of_unnamed_locals, inner_block.named_locals.len());

			crate::alloc_zeroed::<Option<Value>>(layout).as_ptr()
		};

		// Initialize the scratch register to `null`.
		// SAFETY: We know this is in bounds as `num_of_unnamed_locals` is nonzero.
		unsafe {
			unnamed_locals.write(Some(Value::default()));
		}

		// SAFETY
		// - The resulting pointer is in bounds, as we created `unnamed_locals` with at least
		//   `inner_block.num_named_locals`.
		// - `num_of_unnamed_locals` will never reach `isize::MAX`, as per
		//   `block::Builder::unnamed_local`'s safety guarantee.
		// - We don't rely on wrapping behaviour
		let named_locals = unsafe { unnamed_locals.add(inner_block.num_of_unnamed_locals.get()) };

		// Copy the arguments over.
		// SAFETY:
		// - We have allocated enoguh space for all our `write`s, as we allocated enough for
		//   all named locals, which includes `__block__`, `__args__`, as well as normal arguments.
		// - We know that `start` and `args.positional` don't overlap.
		unsafe {
			debug_assert!(inner_block.named_locals.len() >= 2);
			debug_assert!(inner_block.arity <= inner_block.named_locals.len() - 2);

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
		}

		// Fill out and finish the builder
		let mut builder = Base::<Frame>::builder();

		builder.set_parents(block.to_value());
		let data_ptr = builder.data_mut();

		// Fill out the builder
		// SAFETY:
		// - We know `(*data_ptr).xxx` is valid because we got `data_ptr` from `builder`, which we
		//   validly allocated
		unsafe {
			std::ptr::addr_of_mut!((*data_ptr).unnamed_locals).write(unnamed_locals);
			std::ptr::addr_of_mut!((*data_ptr).named_locals).write(named_locals);
			std::ptr::addr_of_mut!((*data_ptr).inner_block).write(inner_block);
			std::ptr::addr_of_mut!((*data_ptr).block).write(block);
		}

		// No need to initialize `pos` as it starts off as zero.
		debug_assert_eq!(unsafe { (*data_ptr).pos }, 0);

		// SAFETY: We've finished creating a valid `Inner`, so we can call `.finish()`.
		Ok(unsafe { builder.finish() })
	}

	/// Fetches the block associated with this stackframe.
	pub fn block(&self) -> Gc<Block> {
		self.block
	}

	pub(crate) fn is_object(&self) -> bool {
		self.flags().contains(FLAG_IS_OBJECT)
	}

	pub(crate) fn convert_to_object(&mut self) -> Result<()> {
		// If we're already an object, nothing else needed to be done.
		if !self.flags().try_acquire_all_user(FLAG_IS_OBJECT) {
			return Ok(());
		}

		let block = self.0.data().block.to_value();

		// Once we start referencing the frame as an object, we no longer can longer use the "block is
		// our only parent" optimization.
		self.set_parents(List::from_slice(&[Gc::<Self>::parent(), block]));

		let (data, mut attrs, _) = self.0.deconstruct_mut();

		// OPTIMIZE: we could use `with_capacity`, but we'd need to move it out of the builder.
		for i in 0..data.inner_block.named_locals.len() {
			// SAFETY:
			// We know that `i` is in bounds b/c we iterate over `data.inner_block.named_locals.len()`.
			if let Some(value) = unsafe { *data.named_locals.add(i) } {
				attrs.set_attr(data.inner_block.named_locals[i].to_value(), value)?;
			}
		}

		Ok(())
	}

	// SAFETY:
	// `index <= self.inner_block.num_of_unnamed_locals` and also have been assigned to.
	unsafe fn get_unnamed_local(&self, index: usize) -> Value {
		debug_assert!(
			index <= self.inner_block.num_of_unnamed_locals.get(),
			"index out of bounds: {index}, where max is {}",
			self.inner_block.num_of_unnamed_locals
		);

		if let Some(value) = *self.unnamed_locals.add(index) {
			value
		} else if cfg!(debug_assertions) {
			unreachable!("reading from an unassigned unnamed local at index {index}??");
		} else {
			// This should never occur, as the bytecode should be well-formed.
			std::hint::unreachable_unchecked()
		}
	}

	// SAFETY:
	// - `index` needs to correspond to a valid named or unnamed index (ie for
	//      `Unnamed`: `<= self.inner_block.num_of_unnamed_locals`,
	//      `Named`:   `<= `self.inner_block.named_locals`
	// - The corresponding value at said index needs to have been assigned to beforehand for unnamed
	//   locals.
	unsafe fn get_local(&self, index: LocalTarget) -> Result<Value> {
		match index {
			LocalTarget::Unnamed(index) => Ok(self.get_unnamed_local(index)),
			LocalTarget::Named(index) => {
				debug_assert!(index <= self.inner_block.named_locals.len());

				if !self.is_object() {
					// Since we could be trying to access a parent scope's variable, we won't return an error
					// in the false case.
					if let Some(value) = *self.named_locals.add(index) {
						return Ok(value);
					}
				}

				// SAFETY: we know `index` is valid, as the caller guarantees it.
				self.get_object_local(index)
			}
		}
	}

	// The vast majority of the time, we're looking for unnamed or named locals, not through parent
	// attributes.
	// SAFETY: `index` has to be a valid named local index.
	#[inline(never)]
	unsafe fn get_object_local(&self, index: usize) -> Result<Value> {
		let attr_name = *self.inner_block.named_locals.get_unchecked(index);

		if let Some(attr) = self.get_unbound_attr_checked(attr_name.to_value(), &mut Vec::new())? {
			return Ok(attr);
		}

		// When we're not an object, we first check the parent block and then the frame.
		// when we are an object, the frame and block (in that order) are checked in the previous
		// function.
		if !self.is_object() {
			if let Some(attr) = Gc::<Self>::parent().get_unbound_attr(attr_name.to_value())? {
				return Ok(attr);
			}
		}

		Err(
			ErrorKind::UnknownAttribute {
				object: crate::value::Gc::new(self.into()).to_value(),
				attribute: attr_name.to_value(),
			}
			.into(),
		)
	}

	// SAFETY: `index` needs to be a valid index.
	unsafe fn set_local(&mut self, index: LocalTarget, value: Value) -> Result<()> {
		match index {
			LocalTarget::Unnamed(index) => {
				debug_assert!(index <= self.inner_block.num_of_unnamed_locals.get());

				self.unnamed_locals.add(index).write(Some(value));

				Ok(())
			}
			LocalTarget::Named(index) => {
				if !self.is_object() {
					self.named_locals.add(index).write(Some(value));
					Ok(())
				} else {
					self.set_object_local(index, value)
				}
			}
		}
	}

	// SAFETY: `index` needs to be a valid index.
	#[inline(never)]
	unsafe fn set_object_local(&mut self, index: usize, value: Value) -> Result<()> {
		let attr_name = *self.inner_block.named_locals.get_unchecked(index);
		self.set_attr(attr_name.to_value(), value)
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

	// SAFETY: `is_done()` should not be true.
	unsafe fn next_byte(&mut self) -> u8 {
		debug_assert!(self.pos + 1 <= self.inner_block.code.len());

		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		let byte = *self.inner_block.code.get_unchecked(self.pos);

		trace!(target: "frame", byte=%format!("{byte:02x}"), sp=%self.pos, "read byte");

		self.pos += 1;
		byte
	}

	// SAFETY: Must be called when there's at least `usize` bytes left.
	#[cold]
	unsafe fn next_usize(&mut self) -> usize {
		debug_assert!(self.pos + std::mem::size_of::<usize>() <= self.inner_block.code.len());

		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		#[allow(clippy::cast_ptr_alignment)]
		let us = self.inner_block.code.as_ptr().add(self.pos).cast::<usize>().read_unaligned();

		self.pos += std::mem::size_of::<usize>();

		us
	}

	// SAFETY: Must be called when there's at least `sizeof(u64)` bytes left.
	unsafe fn next_u64(&mut self) -> u64 {
		debug_assert!(self.pos + std::mem::size_of::<u64>() <= self.inner_block.code.len());

		// SAFETY: `block`s can only be created from well-formed bytecode, so this will never be
		// out of bounds.
		#[allow(clippy::cast_ptr_alignment)]
		let us = self.inner_block.code.as_ptr().add(self.pos).cast::<u64>().read_unaligned();

		self.pos += std::mem::size_of::<u64>();

		us
	}

	// SAFETY: Must be called when the next value is actually a valid local.
	unsafe fn next_local(&mut self) -> Result<Value> {
		let index = self.next_local_target();
		let value = self.get_local(index)?;

		trace!(target: "frame", ?index, ?value, "read local");

		Ok(value)
	}

	// SAFETY: Must be called when there's at least `usize` bytes left.
	unsafe fn next_count(&mut self) -> usize {
		match self.next_byte() {
			COUNT_IS_NOT_ONE_BYTE_BUT_USIZE => self.next_usize(),
			byte if (byte as i8) < 0 => byte as i8 as isize as usize,
			byte => byte as usize,
		}
	}

	// SAFETY: must be called when there's at least `usize` bytes left.
	unsafe fn next_local_target(&mut self) -> LocalTarget {
		match self.next_count() as isize {
			n @ 0.. => LocalTarget::Unnamed(n as usize),
			n => LocalTarget::Named(!n as usize),
		}
	}

	// SAFETY:
	// - At least 1 byte is left
	// - the next byte must be a valid opcode.
	unsafe fn next_opcode(&mut self) -> Opcode {
		let byte = self.next_byte();

		if let Some(op) = Opcode::from_byte(byte) {
			trace!(target: "frame", ?op, "read opcode");
			op
		} else if cfg!(debug_assertions) {
			unreachable!("invalid opcode? {byte:?}")
		} else {
			std::hint::unreachable_unchecked()
		}
	}

	// SAFETY: if we're not at the end, the next byte must be a valid opcode.
	unsafe fn next_op(&mut self) -> Result<Option<Opcode>> {
		if self.is_done() {
			Ok(None)
		} else {
			Ok(Some(self.next_opcode()))
		}
	}

	// SAFETY: index has to be in bounds
	unsafe fn get_constant(&mut self, index: usize, dst: LocalTarget) -> Result<Value> {
		debug_assert!(index <= self.inner_block.constants.len());

		let constant = *self.inner_block.constants.get_unchecked(index);
		if let Some(block) = constant.downcast::<Gc<Block>>() {
			self.constant_as_block(block, dst)
		} else {
			Ok(constant)
		}
	}

	// SAFETY: `dst` must be a valid local target.
	#[inline(never)]
	unsafe fn constant_as_block(&mut self, block: Gc<Block>, dst: LocalTarget) -> Result<Value> {
		self.convert_to_object()?;

		// SAFETY: TODO
		let parent = crate::value::Gc::new(self.into());
		let block = block.as_ref()?.deep_clone_from(parent)?;

		// TODO: maybe pass name to `deep_clone_from` too?
		if let LocalTarget::Named(index) = dst {
			debug_assert!(index <= self.inner_block.named_locals.len());
			let name = *self.inner_block.named_locals.get_unchecked(index);

			debug_assert!(
				block
					.as_ref()
					.unwrap()
					.attributes()
					.get_unbound_attr(Intern::__name__)
					.unwrap()
					.is_none(),
				"somehow assigning a name twice?"
			);

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

	/// Restarts `frame` from the beginning. Note that if it's already running, an error will be
	/// returned.
	pub fn restart(self) -> Result<Value> {
		self.as_mut()?.pos = 0;
		self.run()
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

		match result {
			Ok(()) => self.as_mut().map(|this| {
				// SAFETY: We wrote `null` to the scratch register at creation, so this is guaranteed
				// to always be written to
				unsafe { this.get_unnamed_local(0) }
			}),
			Err(Error { kind: ErrorKind::Return { value, from_frame }, .. })
				if from_frame.map_or(true, |ff| ff.is_identical(self.to_value())) =>
			{
				Ok(value)
			}
			Err(err) => Err(err),
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

		// SAFETY: we're guaranteed the next byte, if it exists, is valid, because `Frame`s can only
		// be created with valid bytecode.
		while let Some(op) = unsafe { this.next_op()? } {
			if cfg!(debug_assertions) {
				for position in args.iter_mut().take(NUM_ARGUMENT_REGISTERS) {
					*position = MaybeUninit::uninit();
				}

				variable_args_count = MaybeUninit::uninit();
			}

			// SAFETY: we're guaranteed the next byte, if it exists, is valid, because `Frame`s can
			// only be created with valid bytecode.
			let dst = unsafe { this.next_local_target() };

			{
				let arity = op.fixed_arity();
				let is_variable_simple = op.is_variable_simple();
				let mut ptr = args.as_mut_ptr().cast::<Value>();

				debug_assert!(arity <= NUM_ARGUMENT_REGISTERS);
				for _ in 0..arity {
					// SAFETY: `ptr` is in bounds, because `arity` is guaranteed to be smaller than
					// `NUM_ARGUMENT_REGISTERS`. Additionally, since `self` is well-formed, we know that
					// the next count is actually a valid local.
					unsafe {
						ptr.write(this.next_local()?);
						ptr = ptr.add(1);
					}
				}

				if is_variable_simple {
					// SAFETY: we're guaranteed the next byte exists, because `Frame`s can only be
					// created with valid bytecode.
					let count = unsafe { this.next_byte() } as usize;
					variable_args_count.write(count);

					// all things with `is_variable` are <= NUM_ARGUMENT_REGISTERS.
					debug_assert_ne!(count, COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize);
					debug_assert!((count as u8 as i8) >= 0);
					debug_assert!(count <= NUM_ARGUMENT_REGISTERS);
					debug_assert!(arity + count <= NUM_ARGUMENT_REGISTERS);

					for _ in 0..count {
						// SAFETY: `ptr` is in bounds, because `arity` is guaranteed to be smaller than
						// `NUM_ARGUMENT_REGISTERS`. Additionally, since `self` is well-formed, we know
						// that the next count is actually a valid local.
						unsafe {
							ptr.write(this.next_local()?);
							ptr = ptr.add(1);
						}
					}
				}
			}

			let result = match op {
				Opcode::CreateList => {
					#[cold]
					fn create_large_list(this: &mut Frame) -> Result<Value> {
						// SAFETY: `self` is well-formed, so after `CreateList` and `dst` follows a count.
						let amnt = unsafe { this.next_count() };

						// TODO: use simple list builder when we make it
						let mut list = List::with_capacity(amnt);

						for _ in 0..amnt {
							// SAFETY: `self` is well-formed, so after `CreateList`'s count is that many
							// locals.
							unsafe {
								list.push_unchecked(this.next_local()?);
							}
						}

						Ok(list.to_value())
					}

					create_large_list(&mut this)?
				}

				Opcode::CreateListSimple => {
					// SAFETY: `self` is well-formed, so we know that `CreateListSimple` is followed by
					// a count, and then a bunch of locals.
					let slice = unsafe {
						std::slice::from_raw_parts(
							args.as_ptr().cast::<Value>(),
							variable_args_count.assume_init(),
						)
					};

					List::from_slice(slice).to_value()
				}

				// SAFETY: `self` is well-formed, so we know the first argument to `Mov` is present
				Opcode::Mov => unsafe { args[0].assume_init() },

				Opcode::Call => todo!(), //self.op_call(),

				// SAFETY: `self` is well-formed, so we know the first argument to `CallSimple` exists,
				// and is followed by a slice of locals.
				Opcode::CallSimple => without_this! {
					unsafe { args[0].assume_init().call(args_slice!(start=1))? }
				},

				// SAFETY: `self` is well-formed, so we know that `ConstLoad`, after `dst`, has a valid
				// count to a constant.
				Opcode::ConstLoad => unsafe {
					let idx = this.next_count();
					this.get_constant(idx, dst)?
				},

				// SAFETY: `self` is well-formed, so we know that `LoadImmediate` will have at least
				// 8 bytes after, corresponding to a valid `Value`.
				Opcode::LoadImmediate => unsafe { Value::from_bits(this.next_u64()) },

				// SAFETY: `self` is well-formed, so we know that `LoadSmallImmediate` will have at
				// least a single byte after, corresponding to a valid `Value`.
				Opcode::LoadSmallImmediate => unsafe {
					Value::from_bits(this.next_byte() as i8 as i64 as u64)
				},

				// SAFETY: `self` is well-formed, so we know that `Gc<Block>` will have at least
				// 8 bytes after, corresponding to a valid `Gc<Block>`.
				Opcode::LoadBlock => unsafe {
					let block = std::mem::transmute::<u64, Gc<Block>>(this.next_u64());
					this.constant_as_block(block, dst)?
				},

				Opcode::Stackframe => {
					#[cold]
					fn stackframe(mut count: isize) -> Result<Gc<Frame>> {
						// todo: optimization for :0
						with_stackframes(|frames| {
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
						})
					}

					// SAFETY: `self` is well-formed, so we know that `Stackframe`, after `dst`, has a
					// valid count.
					let count = unsafe { this.next_count() } as isize;
					let frame = stackframe(count)?;

					without_this! {
						frame.as_mut()?.convert_to_object()?;
					}

					frame.to_value()
				}

				Opcode::GetAttr => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist.
					let (object, attr) = unsafe { (args[0].assume_init(), args[1].assume_init()) };
					object.try_get_attr(attr)?
				},
				Opcode::GetUnboundAttr => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist.
					let (object, attr) = unsafe { (args[0].assume_init(), args[1].assume_init()) };
					object.try_get_unbound_attr(attr)?
				},
				Opcode::HasAttr => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist.
					let (object, attr) = unsafe { (args[0].assume_init(), args[1].assume_init()) };
					object.has_attr(attr)?.to_value()
				},
				Opcode::SetAttr => {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist.
					let (attr, value) = unsafe { (args[0].assume_init(), args[1].assume_init()) };

					// SAFETY: `self` is well-formed, so we know that after the first two arguments is
					// a local target
					let index = unsafe { this.next_local_target() };

					/*
					Because you can assign indices onto any object, we need to be able to dynamically
					convert immediates (eg integers, floats, booleans, etc) into a heap-allocated form if
					we want to assign attributes. This is done by having `Value::set_attr` take a mutable
					reference to self. However, the only time this is useful is if we're talking about a
					named attribute---if we're assigning to an unnamed local, that means it'll just get
					thrown away immediately.

					As such, if it's an unnamed local, we still call the `set_attr`, in case it has a
					side effect, but we don't actually assign the `object` to anything. On the other
					hand, we have to box the `object` if it's not already a box.
					*/
					match index {
						LocalTarget::Unnamed(index) => {
							// SAFETY: `self` is well-formed, so we we're guaranteed `index` is a
							// valid local target.
							let mut object = unsafe { this.get_unnamed_local(index) };

							// the only way for a local to be `self` is if it is an object.
							if cfg!(debug_assertions) && self.to_value().is_identical(object) {
								debug_assert!(this.is_object());
							}

							object.set_attr(attr, value)?;
						}
						LocalTarget::Named(index) => {
							// SAFETY: `self` is well-formed, so we we're guaranteed `index` is a
							// valid local target.
							let name =
								unsafe { this.inner_block.named_locals.get_unchecked(index) }.to_value();
							let object = this.get_unbound_attr_mut(name)?;

							if self.to_value().is_identical(*object) {
								this.convert_to_object()?;
								this.set_attr(attr, value)?;
							} else {
								object.set_attr(attr, value)?;
							}
						}
					}

					value
				}

				Opcode::DelAttr => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist.
					let (mut object, attr) = unsafe { (args[0].assume_init(), args[1].assume_init()) };
					object.del_attr(attr)?.unwrap_or_default()
				},
				Opcode::CallAttr => todo!(),
				Opcode::CallAttrSimple => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first two arguments exist, and
					// are followed by an argument slice
					let (object, attr, args_slice) = unsafe {
						(args[0].assume_init(), args[1].assume_init(), args_slice!(start=2))
					};

					object.call_attr(attr, args_slice)?
				},

				Opcode::Add
				| Opcode::Subtract
				| Opcode::Multiply
				| Opcode::Divide
				| Opcode::Modulo
				| Opcode::Power
				| Opcode::Equal
				| Opcode::NotEqual
				| Opcode::LessThan
				| Opcode::GreaterThan
				| Opcode::LessEqual
				| Opcode::GreaterEqual
				| Opcode::Compare => without_this! {
					static INTERNS_PER_OPCODE_COUNT: [Intern; 13] = [
						Intern::op_add,
						Intern::op_sub,
						Intern::op_mul,
						Intern::op_div,
						Intern::op_mod,
						Intern::op_pow,
						Intern::op_eql,
						Intern::op_neq,
						Intern::op_lth,
						Intern::op_gth,
						Intern::op_leq,
						Intern::op_geq,
						Intern::op_cmp,
					];

					// SAFETY: `self` is well-formed, so we know that the first argument exists, and is
					// followed by an argument slice of length 1.
					let (object, args) = unsafe { (args[0].assume_init(), args_slice!(start=1, len=1)) };

					let opcode_count = op.count_within_arity();
					debug_assert!(opcode_count < INTERNS_PER_OPCODE_COUNT.len());

					// SAFETY: `self` is well-formed, so we know that all of the opcodes matched in this
					// block are <= `INTERNS_PER_OPCODE_COUNT`'s length.
					let intern = unsafe { *INTERNS_PER_OPCODE_COUNT.get_unchecked(opcode_count) };
					object.call_attr(intern, args)?
				},

				Opcode::Not | Opcode::Negate => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first argument exists
					let object = unsafe { args[0].assume_init() };
					let intern = if op == Opcode::Not {
						Intern::op_not
					} else {
						Intern::op_neg
					};
					object.call_attr(intern, Args::default())?
				},
				Opcode::Index | Opcode::IndexAssign => without_this! {
					// SAFETY: `self` is well-formed, so we know that the first argument exists, and is
					// followed by an argument slice.
					let (object, args) = unsafe { (args[0].assume_init(), args_slice!(start=1)) };

					let intern = if op == Opcode::Index {
						Intern::op_index
					} else {
						Intern::op_index_assign
					};

					object.call_attr(intern, args)?
				},
			};

			debug!(target: "frame", ?dst, ?args, ?op, "ran opcode");
			// SAFETY: `self` is well-formed, so we know that `dst` is a valid destination target.
			unsafe {
				this.set_local(dst, result)?;
			}
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
	#[cfg_attr(miri, ignore)]
	fn test_fibonacci() {
		let fib = {
			let mut builder = crate::vm::block::Builder::new(1, Default::default());

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

		assert_eq!(
			result.downcast::<crate::value::ty::Integer>(),
			Some(crate::value::ty::Integer::new(610).unwrap())
		);
	}
}
