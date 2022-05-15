use crate::value::{ToAny, Gc};
use crate::vm::Args;
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Callable(());
}

impl Callable {
	#[must_use]
	pub fn instance() -> AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			new_quest_scope! {
				// "whatever" => Gc::<Callable>::qs_ignore
			}
			.unwrap()
			.to_any()
		})
	}
}

impl Gc<Callable> {

	pub fn qs_ignore(args: Args<'_>) -> Result<AnyValue> {
		let _ = true.to_any();
		let _ = args;
		todo!()
	}
}

quest_type_attrs! { for Gc<Callable>, parent Object;
	// "whatever" => func Gc::<Callable>::qs_ignore
	// in the future, this will be stuff like composing functions.
}
