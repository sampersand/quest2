#[macro_use]
mod macros;

#[macro_use]
pub mod rustfn;

pub mod boolean;
mod boundfn;
mod callable;
pub mod class;
pub mod float;
pub mod integer;
mod kernel;
mod list;
mod null;
pub mod object;
mod pristine;
pub mod scope;
pub mod text;
mod wrap;

pub use boolean::Boolean;
pub use boundfn::BoundFn;
pub use callable::Callable;
pub use class::Class;
pub use float::Float;
pub use integer::Integer;
pub use kernel::Kernel;
pub use list::List;
pub use null::Null;
pub use object::Object;
pub use pristine::Pristine;
pub use rustfn::RustFn;
pub use scope::Scope;
pub use text::Text;
pub use wrap::Wrap;

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

pub trait Singleton: Sized {
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
