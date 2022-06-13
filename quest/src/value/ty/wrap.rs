use crate::value::base::{Base, HasDefaultParent, IntoParent};
use crate::value::gc::Gc;
use std::fmt::{self, Debug, Formatter};

quest_type! {
	pub struct Wrap<T>(T) where {T: 'static};
}

impl<T: HasDefaultParent + 'static> Wrap<T> {
	pub fn new(data: T) -> Gc<Self> {
		Self::with_parent(data, T::parent())
	}
}

impl<T: 'static> Wrap<T> {
	pub fn with_parent<P: IntoParent>(data: T, parent: P) -> Gc<Self> {
		Base::new(data, parent)
	}
}

impl<T: Debug> Debug for Wrap<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_tuple("Wrap").field(&self.0.data()).finish()
	}
}
