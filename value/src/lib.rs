#[macro_use]
extern crate static_assertions;


pub use value::Value;
use base::Allocated;

mod gc;
pub use gc::Gc;

mod err;
pub use err::{Error, Result};

mod attr;
pub use attr::Attributes;
pub mod base;
pub mod kinds;
pub mod value;

pub trait QuestValue : std::fmt::Debug {
	fn parents(&self) -> &[Value];
	fn unique_id(&self) -> u64;
	fn attrs(&self) -> &Attributes;
}

// fn main() {
//     println!("Hello, world!");
// }
