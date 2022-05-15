use crate::value::ty::{self, Integer, Singleton, Text};
use crate::value::{Gc, HasDefaultParent};
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
			print!("{}", *arg.convert::<Gc<Text>>()?.as_ref()?);
		}

		println!();

		Ok(Default::default())
	}

	pub fn dump(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		println!("{:#?}", args[0]);

		Ok(args[0])
	}

	pub fn exit(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		std::process::exit(args[0].convert::<Integer>()? as i32);
	}

	pub fn r#if(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() == 2 || a.positional().len() == 3)?;

		if args[0].is_truthy()? {
			args[1].call(Args::default())
		} else if let Ok(if_false) = args.get(2) {
			if_false.call(Args::default())
		} else {
			Ok(AnyValue::default())
		}
	}

	pub fn r#while(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(2)?;

		let mut last = AnyValue::default();

		while args[0].call(Args::default())?.is_truthy()? {
			last = args[1].call(Args::default())?;
		}

		Ok(last)
	}
}

impl Singleton for Kernel {
	fn instance() -> crate::AnyValue {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Kernel", parent Pristine::instance();
				Intern::print => justargs funcs::print,
				Intern::dump => justargs funcs::dump,
				Intern::exit => justargs funcs::exit,
				Intern::r#if => justargs funcs::r#if,
				Intern::r#while => justargs funcs::r#while,
				Intern::Integer => constant ty::Integer::parent(),
				Intern::r#true => constant true.to_any(),
				Intern::r#false => constant false.to_any(),
				Intern::r#null => constant ty::Null.to_any(),
				// TODO: Other types
			}
		})
	}
}
