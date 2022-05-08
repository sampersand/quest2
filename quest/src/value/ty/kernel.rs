use crate::value::Gc;
use crate::vm::Args;
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Kernel(());
}

pub mod funcs {
	use super::*;

	pub fn print(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;

		for arg in args.positional() {
			print!("{}", *arg.convert::<Gc<crate::value::ty::Text>>()?.as_ref()?);
		}
		println!();

		Ok(Default::default())
	}

	pub fn r#if(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() == 2 || a.positional().len() == 3)?;

		if args[0].is_truthy()? {
			args[1].call(Args::default())
		} else if let Ok(if_false) = args.get(2) {
			if_false.call(Args::default())
		} else {
			Ok(crate::Value::NULL.any())
		}
	}

	pub fn r#while(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;

		let mut last = crate::Value::NULL.any();

		while args[0].is_truthy()? {
			last = args[1].call(Args::default())?;
		}

		Ok(last)
	}
}

singleton_object! { for Kernel, parent Pristine;
	Intern::print => funcs::print,
	Intern::r#if => funcs::r#if,
	Intern::r#while => funcs::r#while,
}
