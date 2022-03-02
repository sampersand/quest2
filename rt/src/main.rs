#![allow(unused)]

#[macro_use]
use qvm_rt;
use qvm_rt::value::ty::*;
use qvm_rt::value::*;

fn foo(x: &[u8]) -> qvm_rt::Result<AnyValue> {
	Ok(Value::from(x[0] as i64 + x[1] as i64).any())
}

fn main() -> qvm_rt::Result<()> {
	let attr = Value::TRUE.any();

	let mut parent = Value::from("hello, world").any();
	parent.set_attr(attr, Value::from(123).any())?;
	assert_eq!(parent.get_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	let mut child = Value::ONE.any();
	assert!(!child.has_attr(attr)?);

	child.parents()?.as_mut()?.push(parent);
	assert_eq!(child.get_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	child.set_attr(attr, Value::from(456).any()).unwrap();
	assert_eq!(child.get_attr(attr)?.unwrap().bits(), Value::from(456).any().bits());

	assert_eq!(child.del_attr(attr)?.unwrap().bits(), Value::from(456).any().bits());
	assert_eq!(child.get_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	Ok(())
}

fn old2(){
	let list = List::from_slice(&[
		Value::from("hello").any(),
		Value::from(12).any(),
		Value::TRUE.any(),
	]);
	let listvalue = Value::from(list).any();

	Gc::get(listvalue.downcast::<Gc<List>>().unwrap())
		.as_mut()
		.unwrap()
		.push(Value::from(12.5).any());

	list.as_mut()
		.unwrap()
		.set_attr(Value::from(0).any(), Value::from("yo").any());

	dbg!(list.as_ref().unwrap().get_attr(Value::from(0).any()));
	
	dbg!(list);
}

fn old() {
	// let rfn = Value::from(qvm_rt::RustFn_new!("foo", foo)).any();
	// dbg!(rfn);

	let mut text1 = Text::from_static_str("Hello, world");
	let text2 = text1.as_ref().unwrap().substr(0..);

	dbg!(text1);
	dbg!(text2);
	assert_eq!(
		text1.as_ref().unwrap().as_ptr(),
		text2.as_ref().unwrap().as_ptr()
	);
	text1.as_mut().unwrap().push('!');
	dbg!(text1); // Hello, world!
	dbg!(text2); // Hello, world
	assert_ne!(
		text1.as_ref().unwrap().as_ptr(),
		text2.as_ref().unwrap().as_ptr()
	);

	println!("{:?}", Value::from(1i64).any());
	println!("{:?}", Value::from(1f64).any());
	println!("{:?}", Value::from(true).any());
	println!("{:?}", Value::from(false).any());
	println!("{:?}", Value::from(Null).any());
	// println!("{:?}", Value::from(base::Base::new(12i64)).any());
	println!("{:?}", Value::from(Text::from_str("yup")).any());
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
