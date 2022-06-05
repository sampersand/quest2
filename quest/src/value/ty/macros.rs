// #[quest::singleton_functions]
// impl Funcs {
// 	fn concat(self, rhs: Merge) -> Result<Value> {
// 		args.assert_no_keyword()?;
// 		args.assert_positional_len(1)?;

// 		let mut rhs = args[0].to_text()?;

// 		if text.ptr_eq(rhs) {
// 			rhs = text.as_ref()?.dup();
// 		}

// 		text.as_mut()?.push_str(rhs.as_ref()?.as_str());

// 		Ok(text.to_value())
// 	}

// 	pub fn add(self, args: Args<'_>) -> Result<Value> {
// 		args.assert_no_keyword()?;
// 		args.assert_positional_len(1)?;

// 		let rhs = args[0].to_text()?;

// 		// TODO: allocate a new string
// 		let text = text.as_ref()?.dup();
// 		text.as_mut().unwrap().push_str(rhs.as_ref()?.as_str());

// 		Ok(text.to_value())
// 	}

// 	pub fn eql(self, args: Args<'_>) -> Result<Value> {
// 		args.assert_no_keyword()?;
// 		args.assert_positional_len(1)?;

// 		if let Some(rhs) = args[0].downcast::<Gc<Text>>() {
// 			Ok((*text.as_ref()? == *rhs.as_ref()?).to_value())
// 		} else {
// 			Ok(false.to_value())
// 		}
// 	}

// 	pub fn len(self, args: Args<'_>) -> Result<Value> {
// 		args.assert_no_arguments()?;
// 		Ok((text.as_ref()?.len() as i64).to_value())
// 	}

// 	pub fn assign(self, args: Args<'_>) -> Result<Value> {
// 		args.assert_no_keyword()?;
// 		args.assert_positional_len(1)?;

// 		let value = args[0];
// 		let mut frame =
// 			crate::vm::frame::with_stackframes(|sfs| *sfs.last().expect("returning from nothing?"))
// 				.to_value();

// 		frame.set_attr(text.to_value(), value)?;

// 		Ok(value)
// 	}

// 	pub fn dbg(self, args: Args<'_>) -> Result<Value> {
// 		args.assert_no_arguments()?;

// 		Ok(Text::from_string(format!("{:?}", text.as_ref()?.as_str())).to_value())
// 	}
// }
