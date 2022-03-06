use crate::value::base::{Builder, HasDefaultParent};
use crate::value::gc::Gc;

quest_type! {
	#[derive(Debug)]
	pub struct Wrap<T>(T) where {T: 'static};
}

impl<T: HasDefaultParent + 'static> Wrap<T> {
	pub fn new(data: T) -> Gc<Self> {
		let mut builder = Builder::<T>::allocate();
		unsafe {
			builder._write_parent(T::parent());
			builder.data_mut().as_mut_ptr().write(data);
			Gc::new(std::mem::transmute(builder.finish()))
		}
	}
}
