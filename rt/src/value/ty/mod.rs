#[macro_use]
pub mod rustfn;

mod boolean;
pub mod float;
pub mod integer;
mod basic;
mod list;
mod null;
pub mod text;
mod block;
mod scope;

pub use boolean::Boolean;
pub use float::Float;
pub use integer::Integer;
pub use list::List;
pub use null::Null;
pub use rustfn::RustFn;
pub use text::Text;
pub use block::Block;
pub use basic::Basic;
pub use scope::Scope;


#[derive(Debug)]
pub struct Wrap<T>(pub T);
impl<T: 'static> crate::value::gc::Allocated for Wrap<T>{}
