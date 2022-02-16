use crate::text::Text;
use qvm_value::*;

fn main() {
	let string = Text::with_capacity(8);
	string.as_mut()
}
