#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct Scope {
	_priv: (),
}

sa::assert_eq_size!(Scope, ());

impl Scope {
	pub const fn new() -> Self {
		Self { _priv: () }
	}
}

// impl Gc<List> {
// 	pub const fn new() -> Self {

// 	}
// }

impl crate::value::base::HasParents for Scope {
	unsafe fn init() {}

	fn parents() -> crate::value::base::Parents {
		Default::default() // todo
	}
}
