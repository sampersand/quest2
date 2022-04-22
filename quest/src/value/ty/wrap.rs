use crate::value::base::{Builder, HasDefaultParent};
use crate::value::gc::Gc;
use std::fmt::{self, Debug, Formatter};

quest_type! {
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

impl<T: Debug> Debug for Wrap<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_tuple("Wrap").field(&self.0.data()).finish()
	}
}
