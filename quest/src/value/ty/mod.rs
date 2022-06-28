#[macro_use]
mod macros;

#[macro_use]
pub mod rustfn;

#[macro_use]
pub mod iterable;

pub mod bignum;
pub mod boolean;
pub mod boundfn;
pub mod callable;
pub mod class;
pub mod float;
pub mod integer;
pub mod iterator;
pub mod kernel;
pub mod list;
pub mod null;
pub mod object;
pub mod pristine;
pub mod scope;
pub mod text;
mod wrap;

pub use bignum::BigNum;
pub use boolean::Boolean;
pub use boundfn::BoundFn;
pub use callable::Callable;
pub use class::Class;
pub use float::Float;
pub use integer::Integer;
pub use iterable::Iterable;
pub use iterator::Iterator;
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
	const ATTR_NAME: crate::Intern;
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
	fn instance() -> crate::Value;
}

pub trait InstanceOf {
	type Parent: Singleton;
}

impl<T: InstanceOf> crate::value::base::HasDefaultParent for T {
	fn parent() -> crate::Value {
		T::Parent::instance()
	}
}
