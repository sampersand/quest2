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

impl Debug for Block {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Block({:?})", self.source_location())
	}
}

#[derive(Debug)]
#[doc(hidden)]
pub struct BlockInner {
	pub(super) arity: usize,
	pub(super) code: Vec<u8>,
	pub(super) location: SourceLocation,
	pub(super) constants: Vec<Value>,
	pub(super) num_of_unnamed_locals: NonZeroUsize,
	pub(super) named_locals: Vec<Gc<Text>>,
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

				if let Some(ref textref) = self.1 {
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

	/// Calls `block` with the given `args`.
	pub fn create_frame(block: Gc<Block>, args: Args<'_>) -> Result<Value> {
		let frame = Frame::new(block, args)?;
		frame.as_mut().unwrap().convert_to_object()?;
		Ok(frame.to_value())
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
