use qvm_value::*;

fn main() {
	let n = Value::from(123f64);

	dbg!(n.any().is_a::<bool>());
}
