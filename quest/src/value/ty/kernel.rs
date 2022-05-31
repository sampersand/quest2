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

		Ok(AnyValue::default())
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

	pub fn abort(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		eprintln!("{}", args[0].convert::<Gc<Text>>()?.as_ref()?.as_str());
		std::process::exit(1);
	}

	pub fn r#if(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() == 2 || a.positional().len() == 3)?;

		if args[0].is_truthy()? {
			args[1].call(Args::default())
		} else if let Some(if_false) = args.get(2) {
			if_false.call(Args::default())
		} else {
			Ok(AnyValue::default())
		}
	}

	pub fn ifl(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() == 2 || a.positional().len() == 3)?;

		if args[0].is_truthy()? {
			Ok(args[1])
		} else {
			Ok(args.get(2).unwrap_or_default())
		}
	}

	pub fn if_cascade(args: Args<'_>) -> Result<AnyValue> {
		args.assert_no_keyword()?;
		args.idx_err_unless(|a| a.positional().len() > 1)?;

		for i in (0..args.len()).step_by(2) {
			if i == args.len() {
				return args[i].call(Args::default());
			}

			if if i == 0 {
				args[i]
			} else {
				args[i].call(Args::default())?
			}.is_truthy()? {
				return args[i+1].call(Args::default())
			}
		}

		Ok(AnyValue::default())
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

	// this isn't the actual interface, im just curious how threads will work out
	pub fn spawn(args: Args<'_>) -> Result<AnyValue> {
		use std::thread::{self, JoinHandle};
		use crate::value::base::Base;
		use crate::value::ty::InstanceOf;
		use crate::value::ToAny;

		quest_type! {
			#[derive(NamedType)]
			pub struct Thread(Option<JoinHandle<Result<AnyValue>>>);
		}

		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
		pub struct ThreadClass;

		impl Singleton for ThreadClass {
			fn instance() -> crate::AnyValue {
				use once_cell::sync::OnceCell;

				static INSTANCE: OnceCell<crate::AnyValue> = OnceCell::new();

				*INSTANCE.get_or_init(|| {
					create_class! { "Thread", parent Object::instance();
						Intern::join => method |thread: Gc<Thread>, args: Args<'_>| -> Result<AnyValue> {
							args.assert_no_arguments()?;

							if let Some(thread) = thread.as_mut()?.0.data_mut().take() {
								thread.join().expect("couldnt join")
							} else {
								Err("unable to join an already join thread".to_string().into())
							}
						},
					}
				})
			}
		}

		impl InstanceOf for Gc<Thread> {
			type Parent = ThreadClass;
		}

		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;
		let func = args[0];

		let thread = thread::spawn(move || func.call(Args::default()));
		Ok(Gc::<Thread>::from_inner(Base::new(Some(thread), Gc::<Thread>::parent())).to_any())
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
				Intern::abort => justargs funcs::abort,
				Intern::r#if => justargs funcs::r#if,
				Intern::ifl => justargs funcs::ifl,
				Intern::if_cascade => justargs funcs::if_cascade,
				Intern::r#while => justargs funcs::r#while,
				Intern::Integer => constant ty::Integer::parent(),
				Intern::List => constant ty::List::parent(),
				// TODO: Other types
				Intern::r#true => constant true.to_any(),
				Intern::r#false => constant false.to_any(),
				Intern::r#null => constant ty::Null.to_any(),

				Intern::spawn => justargs funcs::spawn,
			}
		})
	}
}
