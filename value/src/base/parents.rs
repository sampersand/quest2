use crate::AnyValue;

pub union Parents {
	single: Option<AnyValue>,
}

impl Parents {
	pub const NONE: Self = Self { single: None };
}

pub trait HasParents {
	fn parents() -> Parents;
}
