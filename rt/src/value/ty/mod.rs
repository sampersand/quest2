#[macro_export]
macro_rules! quest_type {
	(
		$(#[$meta:meta])*
		$vis:vis struct $name:ident $(<$($gen:ident),*>)? ($($inner:tt)*) $(where {$($cond:tt)*})?;
	) => {
		$(#[$meta])*
		#[repr(transparent)]
		$vis struct $name $(<$($gen)*>)?($crate::value::base::Base<$($inner)*>) $(where $($cond)*)?;

		unsafe impl $(<$($gen),*>)? $crate::value::gc::Allocated for $name $(<$($gen),*>)?
		$(where $($cond)*)? {
			#[inline(always)]
			fn _inner_typeid() -> ::std::any::TypeId {
				::std::any::TypeId::of::<$($inner)*>()
			}
		}
	};
}

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
mod wrap;

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
pub use wrap::Wrap;
