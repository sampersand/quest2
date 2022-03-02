use crate::value::gc::Gc;
use crate::value::base::{Builder, HasParents};

quest_type! {
	#[derive(Debug)]
	pub struct Wrap<T>(T) where {T: 'static};
}

impl<T: HasParents + 'static> Wrap<T> {
	pub fn new(data: T) -> Gc<Self> {
		let mut builder = Builder::<T>::allocate();
		unsafe {
			builder._write_parents(T::parents());
			builder.data_mut().as_mut_ptr().write(data);
			Gc::new(std::mem::transmute(builder.finish()))
		}
	}
}
