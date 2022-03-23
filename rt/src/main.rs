#![allow(unused)]
#![allow(clippy::all, clippy::nursery, clippy::pedantic)]

#[macro_use]
use qvm_rt;
use qvm_rt::value::ty::*;
use qvm_rt::value::*;
use qvm_rt::vm::Args;
use qvm_rt::Result;
// fn dup(mut obj: AnyValue, _: Args<'_>) -> Result<AnyValue> {
// 	let mut new = obj.downcast::<Gc<Text>>().unwrap().as_ref()?.dup();
// 	new.as_mut()?.parents().as_mut()?.push(obj.parents()?.as_ref()?.as_slice()[0]);

// 	Ok(new.as_any())
// }

fn exclaim(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	obj.call_attr(Intern::concat, Args::new(&["!".as_any()], &[]))?;

	Ok(obj)
}

fn concat(obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
	let lhs = obj.downcast::<Gc<Text>>().unwrap();
	let rhs = args[0].to_text()?;

	lhs.as_mut()?.push_str(rhs.as_ref()?.as_str());

	Ok(lhs.as_any())
}
macro_rules! rustfn {
	($name:ident) => {
		Value::from(qvm_rt::RustFn_new!(stringify!($name), $name)).any()
	};
}

macro_rules! args {
	($($pos:expr),*) => (args!($($pos),* ; ));
	($($kwn:literal => $kwv:expr),*) => (args!(; $($kwn => $kwv),*));
	($($pos:expr),* ; $($kwn:literal => $kwv:expr),*) => {
		Args::new(&[$(value!($pos)),*], &[$(($kwn, value!($kwv))),*])
	}
}

macro_rules! value {
	($lit:literal) => {
		$lit.as_any()
	};
	($name:expr) => {
		$name
	};
}

fn main() -> Result<()> {
	let x = 1
		.as_any()
		.get_attr(Intern::op_add)
		.unwrap()
		.unwrap()
		.get_attr(Intern::op_call)
		.unwrap()
		.unwrap()
		.call_attr(Intern::op_call, args!(2));

	dbg!(x);
	dbg!(Intern::op_call.as_any());

	Ok(())
}

fn main3() -> Result<()> {
	dbg!(1.as_any().call_attr(Intern::op_add, args!(2)));
	dbg!(1
		.as_any()
		.get_unbound_attr(Intern::op_add)
		.unwrap()
		.unwrap()
		.call_attr(Intern::op_call, args![]));
	// main2();
	Ok(())
}

fn main2() -> Result<()> {
	let func1 = qvm_rt::RustFn_new!(
		"func1",
		justargs | args | {
			println!("func1: {:?}", args);

			Ok(args.get_self().unwrap_or_default())
		}
	);

	let func2 = qvm_rt::RustFn_new!(
		"func2",
		justargs | args | {
			println!("func2: {:?}", args);

			Ok(args.get_self().unwrap_or_default())
		}
	);

	// dbg!(Kernel::instance().get_unbound_attr(Intern::r#if)?.unwrap().get_unbound_attr("whatever"));

	println!(
		"result: {:?}",
		Kernel::instance()
			.call_attr(Intern::r#if, Args::new(&[true.as_any(), func1.as_any(), func2.as_any()], &[]))
	);

	println!("{:?}", 1.as_any().call_attr(Intern::op_eql, args!(2)));

	Ok(())
}

fn main1() -> Result<()> {
	{
		// let text_class =
		// 	value!("")
		// 		.get_attr("__parents__")?.unwrap()
		// 		.downcast::<Gc<List>>().unwrap()
		// 		.as_ref()?
		// 		.as_slice()[0];

		// text_class
		// 	.get_attr("__parents__")?.unwrap()
		// 	.downcast::<Gc<List>>().unwrap()
		// 	.as_mut()?
		// 	.push(Pristine::new().as_any());
	}

	let greeting = value!("Hello, world");
	greeting
		.get_attr(Intern::concat)?
		.unwrap()
		.call_attr(Intern::op_call, args!["!"])?;
	greeting.call_attr(Intern::__call_attr__, args!["==", "?"])?;

	println!("{:?}", greeting);
	println!("{:?}", greeting.call_attr(Intern::__call_attr__, args!["len"]));

	Ok(())
}

fn call_attrs() -> Result<()> {
	let greeting = value!("Hello, world");
	greeting.call_attr(Intern::concat, args!["!"])?;
	greeting.call_attr(Intern::concat, args![greeting])?;

	println!("{:?}", greeting); //=> "Hello, world!Hello, world!"
	println!("{:?}", greeting.call_attr(Intern::op_eql, args![greeting])); //=> true
	println!("{:?}", greeting.call_attr(Intern::__call_attr__, args!["==", greeting]));

	let five = value!(5);
	let twelve = value!(12);
	println!("{:?}", five.call_attr(Intern::op_add, args![twelve])); //=> 17

	let ff = value!(255).call_attr(Intern::at_text, args!["base" => 16]);
	println!("{:?}", ff); //=> "ff"

	Ok(())
}

fn get_unbound_attrs() -> Result<()> {
	let attr = Value::TRUE.any();

	let mut parent = Value::from("hello, world").any();
	parent.set_attr(attr, Value::from(123).any())?;
	assert_eq!(parent.get_unbound_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	let mut child = Value::ONE.any();
	assert!(!child.has_attr(attr)?);

	child.parents()?.as_mut()?.push(parent);
	assert_eq!(child.get_unbound_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	child.set_attr(attr, Value::from(456).any()).unwrap();
	assert_eq!(child.get_unbound_attr(attr)?.unwrap().bits(), Value::from(456).any().bits());

	assert_eq!(child.del_attr(attr)?.unwrap().bits(), Value::from(456).any().bits());
	assert_eq!(child.get_unbound_attr(attr)?.unwrap().bits(), Value::from(123).any().bits());

	Ok(())
}

fn lists_work() {
	let list = List::from_slice(&[
		Value::from("hello").any(),
		Value::from(12).any(),
		Value::TRUE.any(),
	]);
	let listvalue = Value::from(list).any();

	listvalue
		.downcast::<Gc<List>>()
		.unwrap()
		.as_mut()
		.unwrap()
		.push(Value::from(12.5).any());

	list
		.as_mut()
		.unwrap()
		.set_attr(Value::from(0).any(), Value::from("yo").any());

	dbg!(list
		.as_ref()
		.unwrap()
		.get_unbound_attr(Value::from(0).any()));

	dbg!(list);
}

fn to_any_works() {
	// let rfn = Value::from(qvm_rt::RustFnnew!("foo", foo)).any();
	// dbg!(rfn);

	let mut text1 = Text::from_static_str("Hello, world");
	let text2 = text1.as_ref().unwrap().substr(0..);

	dbg!(text1);
	dbg!(text2);
	assert_eq!(text1.as_ref().unwrap().as_ptr(), text2.as_ref().unwrap().as_ptr());
	text1.as_mut().unwrap().push('!');
	dbg!(text1); // Hello, world!
	dbg!(text2); // Hello, world
	assert_ne!(text1.as_ref().unwrap().as_ptr(), text2.as_ref().unwrap().as_ptr());

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
