use crate::value::base::HasDefaultParent;
use crate::value::ty::*;
use std::sync::Once;

pub fn init() {
	static START: Once = Once::new();

	START.call_once(|| unsafe { init_quest() })
}

unsafe fn init_quest() {
	// Basic::init();
	Block::init();
	Boolean::init();
	Float::init();
	Integer::init();
	List::init();
	Null::init();
	RustFn::init();
	Scope::init();
	Text::init();
}
