use crate::value::gc::{Allocated};
use crate::value::base::{Base, Header};

#[derive(Debug)]
pub struct Wrap<T: 'static>(Base<T>);

// impl<T: 'static> From<T> for Wrap<T> {
// 	fn from(inp: T) -> Self {
// 		unsafe {
// 			let mut builder = Base::new(inp);
// 		}
// 		Self(inp)
// 	}
// }

impl<T: 'static> Allocated for Wrap<T> {
	fn header(&self) -> &Header {
		self.0.header()
	}

	fn header_mut(&mut self) -> &mut Header {
		self.0.header_mut()
	}
}

