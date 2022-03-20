use crate::value::base::{Builder, HasDefaultParent};
use crate::value::gc::Gc;

quest_type! {
	#[derive(Debug)]
	pub struct Wrap<T>(T) where {T: 'static};
}

impl<T: HasDefaultParent + 'static> Wrap<T> {
	pub fn new(data: T) -> Gc<Self> {
		let mut builder = Builder::<T>::allocate();

		builder.set_parents(T::parent());
		builder.set_data(data);

		Gc::from_inner(unsafe { builder.finish() })
	}
}
