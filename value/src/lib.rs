extern crate static_assertions as sa;


pub use value::Value;
use base::ValueBase;

mod gc;
pub use gc::Gc;
pub mod base;
pub mod text;
pub mod value;

// fn main() {
//     println!("Hello, world!");
// }
