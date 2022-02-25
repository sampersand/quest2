#![allow(unused)]

#[macro_use]
use qvm_rt;
use qvm_rt::value::ty::*;
use qvm_rt::value::*;

fn foo(x: &[u8]) -> qvm_rt::Result<AnyValue> {
	Ok(Value::from(x[0] as i64 + x[1] as i64).any())
}

fn main() {
	let rfn = Value::from(qvm_rt::RustFn_new!("foo", foo)).any();
	dbg!(rfn);


	// let mut text1 = Text::from_static_str("Hello, world");
	// let text2 = text1.as_ref().unwrap().clone();

	// dbg!(text1); // Hello, world
	// dbg!(text2); // Hello, world
	// assert_eq!(text1.as_ref().unwrap().as_ptr(), text2.as_ref().unwrap().as_ptr());
	// text1.as_mut().unwrap().push('!');
	// dbg!(text1); // Hello, world!
	// dbg!(text2); // Hello, world
	// assert_ne!(text1.as_ref().unwrap().as_ptr(), text2.as_ref().unwrap().as_ptr());
	/*

		pub const USER1: u32         = 0b00000000_00000001;
		pub const USER2: u32         = 0b00000000_00000010;
		pub const USER3: u32         = 0b00000000_00000100;
		pub const USER4: u32         = 0b00000000_00001000;

	*/
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
