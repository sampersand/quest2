use crate::value::{AsAny, Gc};
use crate::vm::Args;
use crate::{AnyValue, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Callable1;

impl Callable {
	pub fn instance1() -> AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			new_quest_scope! {
				// "whatever" => Gc::<Callable>::qs_ignore
			}
			.unwrap()
			.as_any()
		})
	}
}

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Callable(());
}

impl Callable {
	pub fn instance() -> AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			new_quest_scope! {
				// "whatever" => Gc::<Callable>::qs_ignore
			}
			.unwrap()
			.as_any()
		})
	}
}

impl Gc<Callable> {
	pub fn qs_ignore(args: Args<'_>) -> Result<AnyValue> {
		let _ = true.as_any();
		let _ = args;
		todo!()
	}
}

quest_type_attrs! { for Gc<Callable>, parent Object;
	// "whatever" => func Gc::<Callable>::qs_ignore
	// in the future, this will be stuff like composing functions.
}
