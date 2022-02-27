use crate::value::gc::Gc;
use crate::value::ty::List;
use crate::value::AnyValue;

#[allow(unused)] // temporarily
pub union Parents {
	single: Option<AnyValue>,
	many: Gc<List>,
}

sa::assert_eq_size!(Parents, u64);
sa::assert_eq_align!(Parents, u64);

impl Parents {
	pub const NONE: Self = Self { single: None };
}

pub trait HasParents {
	fn parents() -> Parents;
}
