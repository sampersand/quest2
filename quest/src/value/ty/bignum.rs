use crate::value::ty::{InstanceOf, Singleton};
use crate::value::{Gc, ToValue};
use crate::vm::Args;
use crate::{Result, Value};
use num_bigint::BigInt;
use std::fmt::{self, Display, Formatter};
use std::ops;

quest_type! {
	#[derive(Debug, NamedType)]
	pub struct BigNum(BigInt);
}

impl BigNum {
	pub fn new(bigint: BigInt) -> Gc<Self> {
		use crate::value::base::{Base, HasDefaultParent};

		Gc::from_inner(Base::new(bigint, Gc::<Self>::parent()))
	}

	pub fn from_i64(num: i64) -> Gc<Self> {
		Self::new(num.into())
	}
}

impl Display for BigNum {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0.data(), f)
	}
}

impl ops::Mul for &BigNum {
	type Output = Gc<BigNum>;

	fn mul(self, rhs: Self) -> Self::Output {
		BigNum::new(self.0.data() * rhs.0.data())
	}
}

pub mod funcs {
	use super::*;

	pub fn at_text(int: Gc<BigNum>, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(int.as_ref()?.to_string().to_value())
		// Ok((int + args[0].try_downcast::<Integer>()?).to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BigNumClass;

impl Singleton for BigNumClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "BigNum", parent Object::instance();
				Intern::at_text => method funcs::at_text
			}
		})
	}
}

impl InstanceOf for Gc<BigNum> {
	type Parent = BigNumClass;
}
