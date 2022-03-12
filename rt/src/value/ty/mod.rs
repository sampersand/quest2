#[macro_use]
mod macros;

#[macro_use]
pub mod rustfn;

mod basic;
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

pub use basic::Basic;
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

pub trait AttrConversionDefined {
	const ATTR_NAME: &'static str;
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
