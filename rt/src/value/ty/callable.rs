use crate::value::{Gc, AsAny};
use crate::{AnyValue, Result};
use crate::vm::Args;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Callable(());
}

impl Callable {
	pub fn instance() -> AnyValue/*Gc<Self>*/ {
		static INSTANCE: once_cell::sync::OnceCell<Gc<Callable>> = once_cell::sync::OnceCell::new();

		INSTANCE.get_or_init(|| {
			use crate::value::base::{Base, HasDefaultParent};

			let inner = Base::new_with_parent((), Gc::<Self>::parent());

			unsafe {
				std::mem::transmute(inner)
			}
		}).as_any()
	}}

impl Gc<Callable> {
	pub fn qs_ignore(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		let _ = true.as_any();
		let _ = (obj, args);
		todo!()
	}
}

quest_type_attrs! { for Gc<Callable>, parent Object;
	// in the future, this will be stuff like composing functions.
}
