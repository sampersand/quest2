use qvm_value::*;
use qvm_value::ty::*;

fn main() {
	let _n = Value::from(123f64);
	let mut builder = Text::builder(100);
	builder.write("Hello");
	builder.write(", ");
	builder.write("world!");
	builder.write("world!");
	builder.write("world!");
	builder.write("world!");
	builder.write("world!");
	builder.write("world!");
	builder.write("world!");
	let text = builder.finish();

	unsafe {
		dbg!(text.as_ref_unchecked());
	}

	// dbg!(n.any().is_a::<bool>());
}
