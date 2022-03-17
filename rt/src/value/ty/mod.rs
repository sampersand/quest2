#[macro_use]
mod macros;

#[macro_use]
pub mod rustfn;

mod kernel;
mod block;
mod boolean;
pub mod float;
pub mod integer;
mod list;
mod null;
pub mod scope;
pub mod text;
mod wrap;
mod boundfn;
mod pristine;
pub mod object;
mod callable;
pub mod class;

pub use kernel::Kernel;
pub use object::Object;
pub use block::Block;
pub use boolean::Boolean;
pub use float::Float;
pub use integer::Integer;
pub use list::List;
pub use null::Null;
pub use rustfn::RustFn;
pub use scope::Scope;
pub use text::Text;
pub use boundfn::BoundFn;
pub use wrap::Wrap;
pub use pristine::Pristine;
pub use callable::Callable;
pub use class::Class;

pub trait AttrConversionDefined {
	const ATTR_NAME: crate::value::Intern;
}

pub trait ConvertTo<T> {
	fn convert(&self, args: crate::vm::Args<'_>) -> crate::Result<T>;
}

impl<T: Clone> ConvertTo<T> for T {
	fn convert(&self, args: crate::vm::Args<'_>) -> crate::Result<T> {
		args.assert_no_arguments()?;

		Ok(self.clone())
	}
}

pub trait Singleton : Sized {
	fn instance() -> crate::AnyValue;
}

pub trait InstanceOf {
	type Parent: Singleton;
}

impl<T: InstanceOf> crate::value::base::HasDefaultParent for T {
	fn parent() -> crate::AnyValue {
		T::Parent::instance()
	}
}
