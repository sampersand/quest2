use super::Block;
use crate::value::ToValue;
use crate::value::{ty::Text, Gc, Value};
use crate::vm::{Opcode, SourceLocation, COUNT_IS_NOT_ONE_BYTE_BUT_USIZE, NUM_ARGUMENT_REGISTERS};
use std::num::NonZeroUsize;

/// A builder for [`Block`].
///
/// This is the only way to create [`Block`]s.
#[derive(Debug, Clone)]
#[must_use]
pub struct Builder {
	arity: usize,
	source_location: SourceLocation,
	code: Vec<u8>,
	constants: Vec<Value>,
	num_of_unnamed_locals: NonZeroUsize,
	named_locals: Vec<Gc<Text>>,
}

impl Default for Builder {
	fn default() -> Self {
		Self::new(0, SourceLocation::default())
	}
}

/// The type of local register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Local {
	/// Can be used by anyone. Also is the return value.
	Scratch,
	/// A local not associated with any variable name.
	Unnamed(usize),
	/// A local with an associated variable name.
	Named(usize),
}

impl Builder {
	/// Creates a new [`Builder`] for a block at the given `source_location`.
	pub fn new(arity: usize, source_location: SourceLocation) -> Self {
		// These are present in every block
		// OPTIMIZE: maybe make these things once and freeze them?
		let named_locals =
			vec![Text::from_static_str("__block__"), Text::from_static_str("__args__")];

		Self {
			arity,
			source_location,
			code: Vec::default(),
			constants: Vec::default(),
			num_of_unnamed_locals: NonZeroUsize::new(1).unwrap(), // The first register is `Scratch`.
			named_locals,
		}
	}

	/// Finish creating the [`Block`].
	#[must_use]
	pub fn build(self) -> Gc<Block> {
		Block::_new(
			self.arity,
			self.code,
			self.source_location,
			self.constants,
			self.num_of_unnamed_locals,
			self.named_locals,
		)
	}

	// SAFETY: you gotta make sure the remainder of the code after this is valid.
	unsafe fn opcode(&mut self, opcode: Opcode, dst: Local) {
		debug!(target: "block_builder", idx=self.code.len(), ?opcode, "set byte");
		self.code.push(opcode as u8);
		self.local(dst);
	}

	// SAFETY: You need to make sure this isn't being used when an `opcode` is expected.
	unsafe fn local(&mut self, local: Local) {
		// debug!(target: "block_builder", "self[{}].local = 0 (scratch)", self.code.len());
		match local {
			Local::Scratch => {
				debug!(target: "block_builder", idx=self.code.len(), local=%"0 (scratch)", "set byte");
				self.code.push(0);
			}
			Local::Unnamed(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				debug!(target: "block_builder", idx=self.code.len(), local=%n, "set byte");
				self.code.push(n as u8);
			}
			Local::Unnamed(n) => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, "set bytes");
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend(n.to_ne_bytes());
			}
			// todo, im not sure if this is 100% correct, math-wise
			Local::Named(n) if n < COUNT_IS_NOT_ONE_BYTE_BUT_USIZE as usize => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, updated=?(!(n as i8) as u8), "set byte");
				self.code.push(!(n as i8) as u8);
			}
			Local::Named(n) => {
				debug!(target: "block_builder", idx=self.code.len(), local=?n, updated=?((!n as isize) as usize), "set bytes");
				self.code.push(COUNT_IS_NOT_ONE_BYTE_BUT_USIZE);
				self.code.extend((!(n as isize)).to_ne_bytes());
			}
		}
	}

	// SAFETY: You need to make sure this isn't being used when an `opcode` is expected.
	unsafe fn count(&mut self, count: usize) {
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

	/// SAFETY: requires `args` and `dst` to correctly correspond to `op`, per `op`'s definition
	pub(crate) unsafe fn simple_opcode<A, I>(&mut self, op: Opcode, dst: Local, args: A)
	where
		A: IntoIterator<Item = Local, IntoIter = I>,
		I: ExactSizeIterator<Item = Local>,
	{
		let args = args.into_iter();
		let len = args.len();

		assert!(
			len <= NUM_ARGUMENT_REGISTERS,
			"too many arguments given for simple_opcode: {len}, max {NUM_ARGUMENT_REGISTERS}",
		);

		self.opcode(op, dst);

		for arg in args {
			self.local(arg);
		}
	}

	/// Creates a new unnamed local.
	///
	/// # Panics
	/// Panics if the amount of unnamed locals is larger than `isize::MAX / 2`.
	pub fn unnamed_local(&mut self) -> Local {
		let num = self.num_of_unnamed_locals.get();
		assert!(num <= (isize::MAX as usize) / 2, "too many unnamed locals");

		self.num_of_unnamed_locals = NonZeroUsize::new(num + 1).unwrap();
		Local::Unnamed(num)
	}

	/// Creates a new named local, returning its previous location if it hadn't existed.
	///
	/// # Panics
	/// Panics if the amount of named locals is larger than `isize::MAX / 2`.
	#[allow(clippy::missing_panics_doc)] // the `.unwrap()` should never panic.
	pub fn named_local(&mut self, name: &str) -> Local {
		for (idx, named_local) in self.named_locals.iter().enumerate() {
			// We created the `Gc<Text>` so no one else should be able to mutate them rn.
			if *named_local.as_ref().unwrap() == name {
				trace!(target: "block_builder", ?idx, ?name, "found named local");
				return Local::Named(idx);
			}
		}

		let idx = self.named_locals.len();
		assert!(idx <= (isize::MAX as usize) / 2, "too many named locals");

		trace!(target: "block_builder", ?idx, ?name, "created new local");

		let named_local = Text::from_str(name);
		named_local.as_ref().unwrap().freeze();
		self.named_locals.push(named_local);
		Local::Named(idx)
	}

	/// Loads the constant `value` into `dst`.
	pub fn constant(&mut self, value: Value, dst: Local) {
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

		// SAFETY: This is the definition of the `ConstLoad` opcode.
		unsafe {
			self.opcode(Opcode::ConstLoad, dst);
			self.count(index);
		}
	}

	/// Equivalent to [`constant`](Self::constant), except there's no need to allocate a
	/// [`Gc<Text>`](crate::value::ty::Text) if the string already exists.
	#[allow(clippy::missing_panics_doc)] // the `.unwrap()` should never panic.
	pub fn str_constant(&mut self, string: &str, dst: Local) {
		let mut index = None;

		// TODO: maybe also check `named_locals`?

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

			self.constants.push(Text::from_str(string).to_value());
			idx
		});

		// SAFETY: This is the definition of the `ConstLoad` opcode.
		unsafe {
			self.opcode(Opcode::ConstLoad, dst);
			self.count(index);
		}
	}

	/// Copies `to` into `from`.
	pub fn mov(&mut self, from: Local, to: Local) {
		// simple short circuit.
		if from == to {
			return;
		}

		// SAFETY: This is the definition of the `Mov` opcode.
		unsafe {
			self.simple_opcode(Opcode::Mov, to, [from]);
		}
	}

	/// Creates a new list from `args`.
	pub fn create_list(&mut self, args: &[Local], dst: Local) {
		// SAFETY: This is the definition of the `CreateList` opcode.
		unsafe {
			self.opcode(Opcode::CreateList, dst);
			self.count(args.len());
			for &arg in args {
				self.local(arg);
			}
		}
	}

	/// <under construction, come back later>
	#[allow(clippy::missing_panics_doc)] // TODO
	pub fn call(&mut self, _dst: Local) {
		// self.opcode(Opcode::Call, dst);
		todo!();
	}

	/// The maximum amount of arguments [`Builder::call_simple`] can take.
	pub const MAX_CALL_SIMPLE_ARGUMENTS: usize = NUM_ARGUMENT_REGISTERS - 1; // `-1` for `what`.

	/// Performs a simple call (ie just positional arguments) of `what` with the arguments `args`,
	/// storing the result into `dst`.
	///
	/// Note that at most [`Builder::MAX_CALL_SIMPLE_ARGUMENTS`] can be passed. If more need to be
	/// passed, instead use [`Builder::call`].
	///
	/// # Panics
	/// Panics if `args.len()` is greater than [`Builder::MAX_CALL_SIMPLE_ARGUMENTS`].
	pub fn call_simple(&mut self, what: Local, args: &[Local], dst: Local) {
		assert!(
			args.len() <= Self::MAX_CALL_SIMPLE_ARGUMENTS,
			"too many arguments given for call_simple: {}, max {}",
			args.len(),
			Self::MAX_CALL_SIMPLE_ARGUMENTS
		);

		// SAFETY: This is the definition of the `CallSimple` opcode.
		unsafe {
			self.opcode(Opcode::CallSimple, dst);
			self.local(what);
			self.count(args.len());
			for &arg in args {
				self.local(arg);
			}
		}
	}

	/// Stores the stackframe at `depth` into` dst`.
	pub fn stackframe(&mut self, depth: isize, dst: Local) {
		// SAFETY: This is the definition of the `Stackframe` opcode.
		unsafe {
			self.opcode(Opcode::Stackframe, dst);
			self.count(depth as usize);
		}
	}

	/// Gets the unbound attribute `attr` from `obj`, storing the result in `dst`.
	pub fn get_unbound_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		// SAFETY: This is the definition of the `GetUnboundAttr` opcode.
		unsafe {
			self.simple_opcode(Opcode::GetUnboundAttr, dst, [obj, attr]);
		}
	}

	/// Gets the attribute `attr` from `obj`, storing the result in `dst`.
	pub fn get_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		// SAFETY: This is the definition of the `GetAttr` opcode.
		unsafe {
			self.simple_opcode(Opcode::GetAttr, dst, [obj, attr]);
		}
	}

	/// Checks to see if `obj` has the attribute `attr`, storing the result in `dst`.
	pub fn has_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		// SAFETY: This is the definition of the `HasAttr` opcode.
		unsafe {
			self.simple_opcode(Opcode::HasAttr, dst, [obj, attr]);
		}
	}

	/// Sets the attribute `attr` on `obj` to `value`, storing `value` back into `dst`.
	pub fn set_attr(&mut self, obj: Local, attr: Local, value: Local, dst: Local) {
		// SAFETY: This is the definition of the `SetAttr` opcode.
		unsafe {
			// NOTE: this puts `obj` last b/c it allows for optimizations on `attr` and `value` parsing
			self.simple_opcode(Opcode::SetAttr, dst, [attr, value, obj]);
		}
	}

	/// Deletes the attribute `attr` from `obj`, storing what was deleted into `dst`
	pub fn del_attr(&mut self, obj: Local, attr: Local, dst: Local) {
		// SAFETY: This is the definition of the `DelAttr` opcode.
		unsafe {
			self.simple_opcode(Opcode::DelAttr, dst, [obj, attr]);
		}
	}

	/// <under construction, come back later>
	#[allow(clippy::missing_panics_doc)] // TODO
	pub fn call_attr(&mut self, _dst: Local) {
		// self.opcode(Opcode::CallAttr, dst);
		todo!();
	}

	/// The maximum amount of arguments [`Builder::call_attr_simple`] can take.
	pub const MAX_CALL_ATTR_SIMPLE_ARGUMENTS: usize = NUM_ARGUMENT_REGISTERS - 2;

	/// Performs a simple attribute call (ie just positional arguments) of `obj`'s attribute `attr`
	/// with the arguments `args`, storing the result into `dst`.
	///
	/// Note that at most [`Builder::MAX_CALL_ATTR_SIMPLE_ARGUMENTS`] can be passed. If more need to
	/// be passed, instead use [`Builder::call_attr`].
	///
	/// # Panics
	/// Panics if `args.len()` is greater than [`Builder::MAX_CALL_ATTR_SIMPLE_ARGUMENTS`].
	pub fn call_attr_simple(&mut self, obj: Local, attr: Local, args: &[Local], dst: Local) {
		assert!(
			args.len() <= Self::MAX_CALL_ATTR_SIMPLE_ARGUMENTS,
			"too many arguments given for call_attr_simple: {}, max {}",
			args.len(),
			Self::MAX_CALL_ATTR_SIMPLE_ARGUMENTS
		);

		// SAFETY: This is the definition of the `CallAttrSimple` opcode.
		unsafe {
			self.opcode(Opcode::CallAttrSimple, dst);
			self.local(obj);
			self.local(attr);
			self.count(args.len());
			for &arg in args {
				self.local(arg);
			}
		}
	}

	/// Adds `lhs` to `rhs`, storing the result into `dst`.
	pub fn add(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Add` opcode.
		unsafe {
			self.simple_opcode(Opcode::Add, dst, [lhs, rhs]);
		}
	}

	/// Subtracts `rhs` from `lhs`, storing the result into `dst`.
	pub fn subtract(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Subtract` opcode.
		unsafe {
			self.simple_opcode(Opcode::Subtract, dst, [lhs, rhs]);
		}
	}

	/// Multiplies `lhs` by `rhs`, storing the result into `dst`.
	pub fn multiply(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Multiply` opcode.
		unsafe {
			self.simple_opcode(Opcode::Multiply, dst, [lhs, rhs]);
		}
	}

	/// Divides `lhs` by `rhs`, storing the result into `dst`.
	pub fn divide(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Divide` opcode.
		unsafe {
			self.simple_opcode(Opcode::Divide, dst, [lhs, rhs]);
		}
	}

	/// Modulos `lhs` by `rhs`, storing the result into `dst`.
	pub fn modulo(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Modulo` opcode.
		unsafe {
			self.simple_opcode(Opcode::Modulo, dst, [lhs, rhs]);
		}
	}

	/// Exponentiates `lhs` by `rhs`, storing the result into `dst`.
	pub fn power(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Power` opcode.
		unsafe {
			self.simple_opcode(Opcode::Power, dst, [lhs, rhs]);
		}
	}

	/// Logically negates `lhs`, storing the result into `dst`.
	pub fn not(&mut self, lhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Not` opcode.
		unsafe {
			self.simple_opcode(Opcode::Not, dst, [lhs]);
		}
	}

	/// Numerically negates `lhs`, storing the result into `dst`.
	pub fn negate(&mut self, lhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Negate` opcode.
		unsafe {
			self.simple_opcode(Opcode::Negate, dst, [lhs]);
		}
	}

	/// Checks to see if `lhs` is equal to `rhs`, storing the result into `dst`.
	pub fn equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Equal` opcode.
		unsafe {
			self.simple_opcode(Opcode::Equal, dst, [lhs, rhs]);
		}
	}

	/// Checks to see if `lhs` is not equal to `rhs`, storing the result into `dst`.
	pub fn not_equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `NotEqual` opcode.
		unsafe {
			self.simple_opcode(Opcode::NotEqual, dst, [lhs, rhs]);
		}
	}

	/// Checks to see if `lhs` is less than `rhs`, storing the result into `dst`.
	pub fn less_than(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `LessThan` opcode.
		unsafe {
			self.simple_opcode(Opcode::LessThan, dst, [lhs, rhs]);
		}
	}

	/// Checks to see if `lhs` is greater than `rhs`, storing the result into `dst`.
	pub fn greater_than(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `GreaterThan` opcode.
		unsafe {
			self.simple_opcode(Opcode::GreaterThan, dst, [lhs, rhs]);
		}
	}

	/// Checks to see if `lhs` is less than or equal to `rhs`, storing the result into `dst`.
	pub fn less_equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `LessEqual` opcode.
		unsafe {
			self.simple_opcode(Opcode::LessEqual, dst, [lhs, rhs]);
		}
	}

	/// Checks to see if `lhs` is greater than or equal to `rhs`, storing the result into `dst`.
	pub fn greater_equal(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `GreaterEqual` opcode.
		unsafe {
			self.simple_opcode(Opcode::GreaterEqual, dst, [lhs, rhs]);
		}
	}

	/// Compares `lhs` and `rhs`, storing the result into `dst`.
	pub fn compare(&mut self, lhs: Local, rhs: Local, dst: Local) {
		// SAFETY: This is the definition of the `Compare` opcode.
		unsafe {
			self.simple_opcode(Opcode::Compare, dst, [lhs, rhs]);
		}
	}

	/// The maximum amount of arguments [`Builder::index`] can take.
	pub const MAX_INDEX_ARGUMENTS: usize = NUM_ARGUMENT_REGISTERS - 1;

	/// Indexes `source` by the arguments `index`, storing the result into `dst`.
	///
	/// Note that at most [`Builder::MAX_INDEX_ARGUMENTS`] can be passed. If more need to be
	/// passed, instead use [`Builder::call_attr`] calling the `[]` attribute instead.
	///
	/// # Panics
	/// Panics if `args.len()` is greater than [`Builder::MAX_INDEX_ARGUMENTS`].
	pub fn index(&mut self, source: Local, index: &[Local], dst: Local) {
		assert!(
			index.len() <= Self::MAX_INDEX_ARGUMENTS,
			"too many arguments for index (got {}, max {}), use call_attr instead",
			index.len(),
			Self::MAX_INDEX_ARGUMENTS,
		);

		// SAFETY: This is the definition of the `Index` opcode.
		unsafe {
			self.opcode(Opcode::Index, dst);
			self.local(source);
			self.count(index.len());
			for &arg in index {
				self.local(arg);
			}
		}
	}

	/// The maximum amount of arguments [`Builder::index_assign`] can take.
	pub const MAX_INDEX_ASSIGN_ARGUMENTS: usize = NUM_ARGUMENT_REGISTERS - 2;

	/// Assigns the index `index` in `source` to `value`, storing `value` back into `dst`.
	///
	/// Note that at most [`Builder::MAX_INDEX_ASSIGN_ARGUMENTS`] can be passed. If more need to be
	/// passed, instead use [`Builder::call_attr`] calling the `[]=` attribute instead.
	///
	/// # Panics
	/// Panics if `args.len()` is greater than [`Builder::MAX_INDEX_ASSIGN_ARGUMENTS`].
	pub fn index_assign(&mut self, source: Local, index: &[Local], value: Local, dst: Local) {
		assert!(
			index.len() <= Self::MAX_INDEX_ASSIGN_ARGUMENTS,
			"too many arguments for index_assign (got {}, max {}), use call_attr instead",
			index.len(),
			Self::MAX_INDEX_ASSIGN_ARGUMENTS,
		);

		// SAFETY: This is the definition of the `IndexAssign` opcode.
		unsafe {
			self.opcode(Opcode::IndexAssign, dst);
			self.local(source);
			self.count(index.len() + 1); // `+1` as `value`'s the last argument.
			for &arg in index {
				self.local(arg);
			}
			self.local(value);
		}
	}
}
