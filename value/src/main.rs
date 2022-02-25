use qvm_value::*;
use qvm_value::ty::*;

fn main() {
	let mut text1 = Text::from_str("Hello, world");
	let text2 = text1.as_ref().unwrap().clone();

	dbg!(text1);
	dbg!(text2);
	text1.as_mut().unwrap().push_str("!");
	dbg!(text1);
	dbg!(text2);
	// assert_eq!(text, text2);

	// println!("{:?}", Value::from(1i64).any());
	// println!("{:?}", Value::from(1f64).any());
	// println!("{:?}", Value::from(true).any());
	// println!("{:?}", Value::from(false).any());
	// println!("{:?}", Value::from(Null).any());
	// println!("{:?}", Value::from(base::Base::new(12)).any());
	// println!("{:?}", Value::from(Text::from_str("yup")).any());
/*	
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
	let value = Value::from(text);

	dbg!(value.any().is_a::<Gc<Text>>());
	dbg!(value.any().downcast::<Gc<Text>>().is_some());

	unsafe {
		dbg!(text.as_ref_unchecked());
	}

	// dbg!(n.any().is_a::<bool>());
	*/
}
