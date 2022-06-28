use crate::value::ty::{InstanceOf, Singleton, Text};
use crate::value::{Gc, ToValue};
use crate::vm::Args;
use crate::{Result, Value};
use std::fmt::{self, Debug, Formatter};

#[macro_export]
macro_rules! iterator {
	($name:expr; $($body:tt)*) => {
		$crate::value::ty::Iterator::new(
			$name,
			move |args: $crate::vm::Args<'_>| -> $crate::Result<$crate::Value> {
				#[allow(unused_imports)]
				use $crate::ErrorKind::StopIteration;
				args.assert_no_arguments()?;
				$($body)*
			}
		)
	};
}

quest_type! {
	#[derive(NamedType)]
	pub struct Iterator(Inner);
}

#[doc(hidden)]
pub struct Inner {
	name: &'static str,
	function: Box<dyn FnMut(Args<'_>) -> Result<Value>>,
}

impl Debug for Iterator {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Iterator").field("name", &self.0.data().name).finish()
		} else {
			f.debug_tuple("Iterator").field(&self.0.data().name).finish()
		}
	}
}

impl Iterator {
	#[must_use]
	pub fn new(
		name: &'static str,
		function: impl FnMut(Args<'_>) -> Result<Value> + 'static,
	) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Base::new(Inner { name, function: Box::new(function) }, Gc::<Self>::parent())
	}

	pub fn empty(name: &'static str) -> Gc<Self> {
		iterator! { name; Err(StopIteration.into()) }
	}

	pub fn next(&mut self, args: Args<'_>) -> Result<Value> {
		(self.0.data_mut().function)(args)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IteratorClass;

impl Singleton for IteratorClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Iterator", parent Iterable::instance();
				Intern::next => method funcs::next,
				Intern::dbg => method funcs::dbg,
			}
		})
	}
}

impl InstanceOf for Gc<Iterator> {
	type Parent = IteratorClass;
}

pub mod funcs {
	use super::*;

	pub fn next(obj: Gc<Iterator>, args: Args<'_>) -> Result<Value> {
		obj.as_mut()?.next(args)
	}

	pub fn dbg(iterator: Gc<Iterator>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		// TODO: maybe cache this in the future?

		let mut builder = Text::simple_builder();

		builder.push_str("<Iterator:");
		builder.push_str(&format!("{iterator:p}"));
		builder.push(':');
		builder.push_str(iterator.as_ref()?.0.data().name);
		builder.push('>');

		Ok(builder.finish().to_value())
	}
}
