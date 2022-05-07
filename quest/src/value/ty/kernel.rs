use crate::value::Gc;
use crate::vm::Args;
use crate::{AnyValue, Result};

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct Kernel(());
}

impl Gc<Kernel> {
	pub fn qs_print(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;

		for arg in args.positional() {
			print!("{}", *arg.convert::<Gc<crate::value::ty::Text>>()?.as_ref()?);
		}
		println!();

		Ok(Default::default())
	}

	pub fn qs_if(args: Args<'_>) -> Result<AnyValue> {
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

	pub fn qs_while(args: Args<'_>) -> Result<AnyValue> {
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
	Intern::print => Gc::<Kernel>::qs_print,
	Intern::r#if => Gc::<Kernel>::qs_if,
	Intern::r#while => Gc::<Kernel>::qs_while,
}
