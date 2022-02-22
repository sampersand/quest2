#![allow(unused)]

use crate::list::List;
use crate::text::Text;
use qvm_value::*;

fn main() {
	let mut list = List::with_capacity(1);

	{
		let mut l = list.as_mut().unwrap();
		l.push(12.into());
		l.push(true.into());
		l.push(false.into());
		l.push((-12.34).into());
		l.extend_from_slice(&[Value::NULL, "foo".into()]);
	}

	dbg!(-12.34f32);
	dbg!(&list.as_ref().unwrap());
}
