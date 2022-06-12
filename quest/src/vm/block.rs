//! Types relating to Quest [`Block`]s.
use super::{Frame, SourceLocation};
use crate::value::gc::{Allocated, Gc};
use crate::value::ty::{List, Text};
use crate::value::{base::Base, HasDefaultParent, Intern, ToValue};
use crate::vm::Args;
use crate::{Result, Value};
use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroUsize;
use std::sync::Arc;

mod builder;
pub use builder::{Builder, Local};

quest_type! {
	/// Represents a block (ie anonymous function) within Quest.
	#[derive(NamedType)]
	pub struct Block(Arc<BlockInner>);
}

#[doc(hidden)]
pub struct BlockInner {
	pub(super) arity: usize,
	pub(super) location: SourceLocation,
	pub(super) named_locals: Vec<Gc<Text>>,
	pub(super) code: Vec<u8>,
	pub(super) constants: Vec<Value>,
	pub(super) num_of_unnamed_locals: NonZeroUsize,
}

impl Debug for Block {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if !f.alternate() {
			return write!(f, "Block({:?})", self.source_location());
		}

		let inner = self.inner();
		f.debug_struct("Block")
			.field("arity", &inner.arity)
			.field("location", &inner.location)
			.field("code", &CodeDebugger(&inner))
			.finish()
	}
}

impl Block {
	fn _new(
		arity: usize,
		code: Vec<u8>,
		location: SourceLocation,
		constants: Vec<Value>,
		num_of_unnamed_locals: NonZeroUsize,
		named_locals: Vec<Gc<Text>>,
	) -> Gc<Self> {
		let inner = Arc::new(BlockInner {
			arity,
			code,
			location,
			constants,
			num_of_unnamed_locals,
			named_locals,
		});

		Gc::from_inner(Base::new(inner, Gc::<Self>::parent()))
	}

	pub(crate) fn inner(&self) -> Arc<BlockInner> {
		self.0.data().clone()
	}

	pub fn arity(&self) -> usize {
		self.0.data().arity
	}

	/// Gets the place that `self` was defined.
	pub fn source_location(&self) -> &SourceLocation {
		&self.0.data().location
	}

	/// Sets the name associated with this block.
	///
	/// # Errors
	/// Returns any errors associated with [setting attributes](Value::set_attr).
	pub fn set_name(&mut self, name: Gc<Text>) -> Result<()> {
		self.header_mut().set_attr(Intern::__name__, name.to_value())
	}

	/// Fetches the name associated with this block, if it exists.
	///
	/// # Errors
	/// Returns any errors associated with [setting attributes](Value::set_attr).
	pub fn name(&self) -> Result<Option<Gc<Text>>> {
		Ok(self
			.header()
			.attributes()
			.get_unbound_attr(Intern::__name__)?
			.and_then(|x| x.downcast::<Gc<Text>>()))
	}

	/// Gets a displayable version of `self`.
	///
	/// This returns a [`Result`] because [`Block::name`] can error.
	///
	/// # Errors
	/// Returns any errors caused by [`Block::name`].
	pub fn display(&self) -> Result<impl Display + '_> {
		struct BlockDisplay<'a>(&'a SourceLocation, Option<crate::value::gc::Ref<Text>>);

		impl Display for BlockDisplay<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				write!(f, "{} ", self.0)?;

				if let Some(textref) = self.1.as_ref() {
					Display::fmt(&**textref, f)
				} else {
					f.write_str("<unnamed>")
				}
			}
		}

		let source_location = self.source_location();
		let name = if let Some(name) = self.name()? { Some(name.as_ref()?) } else { None };

		Ok(BlockDisplay(source_location, name))
	}

	/// Deep clones `self`, returning a completely independent copy, and adding `frame` as a parent
	///
	/// # Errors
	/// Returns any errors associated with [setting attributes](Value::set_attr).
	pub(super) fn deep_clone_from(&self, parent_scope: Gc<Frame>) -> Result<Gc<Self>> {
		#[cfg(debug_assertions)] // needed otherwise `_is_just_single_and_identical` isnt defined?
		debug_assert!(self.header().parents()._is_just_single_and_identical(Gc::<Self>::parent()));

		// TODO: optimize me, eg maybe have shared attributes pointer or something
		let inner = self.inner();
		let parents = List::from_slice(&[Gc::<Self>::parent(), parent_scope.to_value()]);
		// this
		let cloned = Gc::<Self>::from_inner(Base::new(inner, parents));

		let mut clonedmut = cloned.as_mut().unwrap();
		for (attr, value) in self.header().attributes().iter() {
			clonedmut.set_attr(attr, value)?;
		}

		Ok(cloned)
	}
}

impl Gc<Block> {
	/// Executes the block.
	///
	/// This is a convenience wrapper around [`Frame::new`] and [`Gc<Frame>::run`].
	///
	/// # Errors
	/// Returns any errors caused by [`Gc<Frame>::run`].
	pub fn run(self, args: Args<'_>) -> Result<Value> {
		Frame::new(self, args)?.run()
	}

	/// Creates, but doesnt execute, a frame for `self`
	pub fn create_frame(self, args: Args<'_>) -> Result<Gc<Frame>> {
		let frame = Frame::new(self, args)?;

		// We have to make it an object, as otherwise we wont be able to access
		// its local variables.
		frame.as_mut().unwrap().convert_to_object()?;

		Ok(frame)
	}
}

/// Quest functions defined for [`Block`].
#[allow(clippy::missing_errors_doc)]
pub mod funcs {
	use super::*;
	use crate::value::ToValue;

	/// Calls `block` with the given `args`.
	pub fn call(block: Gc<Block>, args: Args<'_>) -> Result<Value> {
		block.run(args)
	}

	/// Creates, but doesnt execute, a frame for `block`
	pub fn create_frame(block: Gc<Block>, args: Args<'_>) -> Result<Value> {
		block.create_frame(args).map(ToValue::to_value)
	}

	/// Returns a debug representation of `block`.
	pub fn dbg(block: Gc<Block>, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;

		let blockref = block.as_ref()?;

		// TODO: maybe cache this in the future?
		let mut builder = Text::simple_builder();
		builder.push_str("<Block");
		if let Some(name) = blockref.name()? {
			builder.push(':');
			builder.push_str(name.as_ref()?.as_str());
		}
		builder.push('@');
		builder.push_str(&blockref.inner().location.to_string());
		builder.push('>');

		Ok(builder.finish().to_value())
	}
}

quest_type_attrs! { for Gc<Block>, parent Object;
	op_call => meth funcs::call,
	create_frame => meth funcs::create_frame,
	dbg => meth funcs::dbg,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}

struct CodeDebugger<'a>(&'a BlockInner);

impl Debug for CodeDebugger<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		struct LclDbg<'a>(isize, &'a BlockInner);
		impl Debug for LclDbg<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				Display::fmt(self, f)
			}
		}

		impl Display for LclDbg<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if self.0 < 0 {
					write!(
						f,
						"{} ({})",
						self.0,
						*self.1.named_locals[!self.0 as usize].as_ref().unwrap()
					)
				} else {
					write!(f, "{}", self.0)
				}
			}
		}

		use crate::vm::Opcode;

		let mut i = 0;
		let mut len;

		macro_rules! byte {
			() => {{
				let byte = self.0.code[i];
				i += 1;
				len += 3;
				write!(f, "{byte:02x} ")?;
				byte
			}};
		}

		macro_rules! u64 {
			() => {{
				let bytes = &self.0.code[i..i + std::mem::size_of::<u64>()];
				i += std::mem::size_of::<u64>();

				for byte in bytes {
					len += 3;
					write!(f, "{byte:02x} ")?;
				}

				u64::from_ne_bytes(bytes.try_into().unwrap())
			}};
		}

		macro_rules! usize {
			() => {{
				let bytes = &self.0.code[i..i + std::mem::size_of::<usize>()];
				i += std::mem::size_of::<usize>();

				for byte in bytes {
					write!(f, "{byte:02x} ")?;
				}

				usize::from_ne_bytes(bytes.try_into().unwrap())
			}};
		}

		macro_rules! local {
			() => {
				LclDbg(count!() as isize, self.0)
			};
		}

		macro_rules! count {
			() => {
				match byte!() {
					super::COUNT_IS_NOT_ONE_BYTE_BUT_USIZE => usize!(),
					byte if (byte as i8) < 0 => byte as i8 as usize,
					byte => byte as usize,
				}
			};
		}

		macro_rules! writeln_len {
			($($tt:tt)*) => {{
				for _ in 0..(30-len) {
					write!(f, " ")?;
				}
				writeln!($($tt)*)
			}};
		}

		let mut amnt_of_opcodes = 0;
		f.write_str("{\n")?;
		while i < self.0.code.len() {
			amnt_of_opcodes += 1;
			f.write_str("\t")?;
			len = 0;

			let op = Opcode::from_byte(byte!()).expect("bad opcode");
			let dst = local!();

			match op {
				Opcode::CreateList => {
					let count = count!();
					let mut list = Vec::with_capacity(count);
					for _ in 0..count {
						list.push(local!());
					}
					writeln_len!(f, "CreateList: dst={dst}, list={list:?}")?;
				}
				Opcode::CreateListSimple => {
					let count = count!();
					let mut list = Vec::with_capacity(count as usize);
					for _ in 0..count {
						list.push(local!());
					}
					writeln_len!(f, "CreateListSimple: dst={dst}, list={list:?}")?;
				}

				Opcode::ConstLoad => {
					let idx = count!();
					writeln_len!(
						f,
						"ConstLoad: dst={dst}, idx={idx} {{{:?}}}",
						self.0.constants[idx as usize]
					)?
				}
				Opcode::LoadSmallImmediate => {
					let bits = byte!() as i8 as i64 as u64;
					let immediate = unsafe { <Value>::from_bits(bits) };
					writeln_len!(f, "LoadSmallImmediate: dst={dst}, immediate={immediate:?}")?;
				}

				Opcode::LoadImmediate => {
					let bits = u64!();
					let immediate = unsafe { <Value>::from_bits(bits) };
					writeln_len!(f, "LoadImmediate: dst={dst}, immediate={immediate:?}")?;
				}
				Opcode::LoadBlock => {
					let bits = u64!();
					let block = unsafe { std::mem::transmute::<u64, Gc<Block>>(bits) };
					writeln_len!(f, "LoadBlock: dst={dst}, block={block:?}")?;
				}
				Opcode::Stackframe => {
					let count = count!();
					writeln_len!(f, "Stackframe: dst={dst}, count={count}")?;
				}

				Opcode::Mov => {
					let src = local!();
					writeln_len!(f, "Mov: dst={dst}, src={src}")?;
				}
				Opcode::Call => todo!(),
				Opcode::CallSimple | Opcode::Index | Opcode::IndexAssign => {
					let obj = local!();
					let count = count!();
					let mut args = Vec::with_capacity(count as usize);
					for _ in 0..count {
						args.push(local!());
					}
					writeln_len!(f, "{op:?}: dst={dst}, obj={obj}, args={args:?}")?;
				}
				Opcode::Not | Opcode::Negate => {
					let src = local!();
					writeln_len!(f, "{op:?}: dst={dst}, src={src}")?;
				}

				Opcode::GetAttr | Opcode::GetUnboundAttr | Opcode::HasAttr | Opcode::DelAttr => {
					let obj = local!();
					let attr = local!();
					writeln_len!(f, "{op:?}: dst={dst}, obj={obj}, attr={attr}")?;
				}
				Opcode::SetAttr => {
					let attr = local!();
					let value = local!();
					let obj = local!();
					writeln_len!(f, "{op:?}: dst={dst}, obj={obj}, attr={attr}, value={value}")?;
				}

				Opcode::CallAttr => todo!(),
				Opcode::CallAttrSimple => {
					let obj = local!();
					let attr = local!();
					let count = count!();
					let mut args = Vec::with_capacity(count as usize);
					for _ in 0..count {
						args.push(local!());
					}
					writeln_len!(f, "{op:?}: dst={dst}, obj={obj}, attr={attr}, args={args:?}")?;
				}
				Opcode::Add
				| Opcode::Subtract
				| Opcode::Multiply
				| Opcode::Divide
				| Opcode::Modulo
				| Opcode::Power
				| Opcode::Equal
				| Opcode::NotEqual
				| Opcode::LessThan
				| Opcode::LessEqual
				| Opcode::GreaterThan
				| Opcode::GreaterEqual
				| Opcode::Compare => {
					let lhs = local!();
					let rhs = local!();
					writeln_len!(f, "{op:?}: dst={dst}, lhs={lhs}, rhs={rhs}")?;
				}
			}
		}

		write!(f, "num_opcodes={amnt_of_opcodes}")?;
		f.write_str("}")
	}
}
