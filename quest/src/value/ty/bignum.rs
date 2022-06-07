use crate::value::ty::{InstanceOf, Singleton};
use crate::value::{Gc, ToValue};
use crate::vm::Args;
use crate::{ErrorKind, Result, Value};
use num_bigint::BigInt;
use num_traits::identities::Zero;
use std::fmt::{self, Display, Formatter};

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

	pub fn checked_add(&self, rhs: &Self) -> Gc<Self> {
		Self::new(self.as_ref() + rhs.as_ref())
	}

	pub fn checked_sub(&self, rhs: &Self) -> Gc<Self> {
		Self::new(self.as_ref() - rhs.as_ref())
	}

	pub fn checked_mul(&self, rhs: &Self) -> Gc<Self> {
		Self::new(self.as_ref() * rhs.as_ref())
	}

	pub fn checked_div(&self, rhs: &Self) -> Result<Gc<Self>> {
		if rhs.as_ref().is_zero() {
			return Err(ErrorKind::DivisionByZero("division").into());
		}

		Ok(Self::new(self.as_ref() / rhs.as_ref()))
	}
}

impl ToValue for BigInt {
	fn to_value(self) -> Value {
		panic!();
		BigNum::new(self).to_value()
	}
}

impl Display for BigNum {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.0.data(), f)
	}
}

impl AsRef<BigInt> for BigNum {
	fn as_ref(&self) -> &BigInt {
		self.0.data()
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
