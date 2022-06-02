//! Types relating to Quest [`Block`]s.

use super::{Frame, SourceLocation};
use crate::value::ty::{List, Text};
use crate::value::{base::Base, Intern, HasDefaultParent, ToAny};
use crate::value::gc::{Gc, Allocated};
use crate::vm::Args;
use crate::{AnyValue, Result};
use std::fmt::{self, Debug, Display, Formatter};
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
	pub(super) code: Vec<u8>,
	pub(super) location: SourceLocation,
	pub(super) constants: Vec<AnyValue>,
	pub(super) num_of_unnamed_locals: usize,
	pub(super) named_locals: Vec<Gc<Text>>,
}

impl Block {
	fn _new(
		code: Vec<u8>,
		location: SourceLocation,
		constants: Vec<AnyValue>,
		num_of_unnamed_locals: usize,
		named_locals: Vec<Gc<Text>>,
	) -> Gc<Self> {
		let inner = Arc::new(BlockInner {
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

	/// Gets the place that `self` was defined.
	pub fn source_location(&self) -> &SourceLocation {
		&self.0.data().location
	}

	/// Sets the name associated with this block.
	pub fn set_name(&mut self, name: Gc<Text>) -> Result<()> {
		debug_assert!(
			self.header()
				.attributes()
				.get_unbound_attr(Intern::__name__)
				.unwrap()
				.is_none(),
				"somehow assigning a name twice?"
		);
		self.header_mut().set_attr(Intern::__name__, name.to_any())
	}

	/// Fetches the name associated with this block, if it exists.
	///
	/// This returns a [`Result`] because attribute access can error.
	pub fn name(&self) -> Result<Option<Gc<Text>>> {
		Ok(self.header()
			.attributes()
			.get_unbound_attr(Intern::__name__)?
			.and_then(|x| x.downcast::<Gc<Text>>()))

	}

	/// Gets a displayable version of `self`.
	///
	/// This returns a [`Result`] because [`Block::name`] can error.
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
		let name = if let Some(name) = self.name()? {
			Some(name.as_ref()?)
		} else {
			None
		};

		Ok(BlockDisplay(source_location, name))
	}

	/// Deep clones `self`, returning a completely independent copy, and adding `frame` as a parent
	pub fn deep_clone_from(&self, parent_scope: Gc<Frame>) -> Result<Gc<Self>> {
		debug_assert!(self.header().parents()._is_just_single_and_identical(Gc::<Self>::parent()));

		// TODO: optimize me, eg maybe have shared attributes pointer or something
		let inner = self.inner();
		let parents = List::from_slice(&[Gc::<Self>::parent(), parent_scope.to_any()]);
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
	/// This is a convenience wrapper around [`Frame::new`] and [`Frame::run`].
	pub fn run(self, args: Args<'_>) -> Result<AnyValue> {
		Frame::new(self, args)?.run()
	}
}

/// Quest functions defined for [`Block`].
pub mod funcs {
	use super::*;
	use crate::value::ToAny;

	/// Calls `block` with the given `args`.
	pub fn call(block: Gc<Block>, args: Args<'_>) -> Result<AnyValue> {
		block.run(args)
	}

	/// Returns a debug representation of `block`.
	pub fn dbg(block: Gc<Block>, args: Args<'_>) -> Result<AnyValue> {
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

		Ok(builder.finish().to_any())
	}
}

quest_type_attrs! { for Gc<Block>, parent Object;
	op_call => meth funcs::call,
	dbg => meth funcs::dbg,
	// "+" => meth qs_add,
	// "@text" => meth qs_at_text,
}
