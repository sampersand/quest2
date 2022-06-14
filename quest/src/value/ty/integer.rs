use crate::value::ty::{ConvertTo, Float, InstanceOf, List, Singleton, Text};
use crate::value::{Convertible, Gc};
use crate::vm::Args;
use crate::{ErrorKind, Intern, Result, ToValue, Value};
use num_bigint::BigInt;
use std::fmt::{self, Debug, Display, Formatter};

pub type Inner = i64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Integer {
	// We have it as `n` instead of `0` to make it harder to accidentally create it.
	n: Inner,
}

impl PartialEq<Inner> for Integer {
	fn eq(&self, rhs: &Inner) -> bool {
		self.get() == *rhs
	}
}

impl Integer {
	/// The largest possible [`Integer`]. Anything larger than this will become a [`BigNum`].
	pub const MAX: Self = unsafe { Self::new_raw(i64::MAX & !1) };

	/// The smallest possible [`Integer`]. Anything smaller than this will become a [`BigNum`].
	pub const MIN: Self = unsafe { Self::new_raw(i64::MIN & !1) };
	// pub const MAX: Self = Self(((u64::MAX >> 2) as Inner) << 1); // all but the top two bits

	// /// The smallest possible [`Integer`]. Anything smaller than this will become a [`BigNum`].
	// pub const MIN: Self = Self((!Self::MAX.0) & !1);

	pub const ONE: Self = Self::new_truncate(1);
	pub const NEG_ONE: Self = Self::new_truncate(-1);
	pub const ZERO: Self = Self::new_truncate(0);

	pub const fn new(num: Inner) -> Option<Self> {
		let truncated = Self::new_truncate(num);

		if truncated.get() == num {
			Some(Self::new_truncate(num))
		} else {
			None
		}
	}

	pub const unsafe fn new_raw(num: Inner) -> Self {
		debug_assert!(num & 1 == 0);
		Self { n: num }
	}

	pub const fn get_raw(self) -> i64 {
		self.n
	}

	pub const fn new_truncate(num: Inner) -> Self {
		Self { n: num << 1 }
	}

	pub const fn get(self) -> Inner {
		debug_assert!(self.n & 1 == 0);

		self.n >> 1
	}
}

impl ToValue for Inner {
	// soft deprecated
	#[must_use]
	fn to_value(self) -> Value {
		Integer::new_truncate(self).to_value()
	}
}

impl Debug for Integer {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.get(), f)
	}
}

impl Display for Integer {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&self.get(), f)
	}
}

impl crate::value::NamedType for Integer {
	const TYPENAME: crate::value::Typename = "Integer";
}

impl Value<Integer> {
	pub const ZERO: Self = unsafe { Self::from_bits(0b0000_0001) };
	pub const ONE: Self = unsafe { Self::from_bits(0b0000_0011) };
}

impl From<Integer> for Value<Integer> {
	#[inline]
	fn from(integer: Integer) -> Self {
		debug_assert_ne!(integer.n & 1, 1);
		let bits = integer.n | 1;

		unsafe { Self::from_bits(bits as _) }
	}
}

unsafe impl Convertible for Integer {
	#[inline]
	fn is_a(value: Value) -> bool {
		(value.bits() & 1) == 1
	}

	fn get(value: Value<Self>) -> Self {
		let bits = (value.bits() as Inner) - 1;
		unsafe { Self::new_raw(bits) }
	}
}

impl super::AttrConversionDefined for Integer {
	const ATTR_NAME: Intern = Intern::to_int;
}

impl ConvertTo<Gc<Text>> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Gc<Text>> {
		args.assert_no_positional()?;

		let base = if let Some(base) = args.get("base") {
			args.idx_err_unless(|_| args.len() == 1)?;
			base.to_integer()?.get()
		} else {
			args.idx_err_unless(Args::is_empty)?;
			10
		};

		if (2..=36).contains(&base) {
			Ok(Text::from_string(radix_fmt::radix(self.get(), base as u8).to_string()))
		} else {
			Err(format!("invalid radix '{base}'").into())
		}
	}
}

impl ConvertTo<Float> for Integer {
	fn convert(&self, args: Args<'_>) -> Result<Float> {
		args.assert_no_arguments()?;

		#[allow(clippy::cast_precision_loss)] // Literally the definition of this method.
		Ok(self.get() as Float)
	}
}

impl Integer {
	pub fn checked_neg(self) -> Value {
		// TODO
		Self::ZERO.checked_sub(self)
	}

	#[quest_fn(name)]
	pub fn op_add(self, rhs: Integer) -> Result<Value> {
		Ok(self.checked_add(rhs))
	}

	// #[quest_fn(name = "op_add")]
	pub fn checked_add(self, rhs: Self) -> Value {
		if let Some(integer) = self.n.checked_add(rhs.n) {
			// SAFETY: It's impossible to add even `i64`s and get an odd one.
			return unsafe { Self::new_raw(integer) }.to_value();
		}

		(self.to_bigint() + &rhs.get()).to_value()
	}

	pub fn checked_sub(self, rhs: Self) -> Value {
		if let Some(integer) = self.n.checked_sub(rhs.n) {
			// SAFETY: It's impossible to subtract even `i64`s and get an odd one.
			return unsafe { Self::new_raw(integer & !1) }.to_value();
		}

		(self.to_bigint() - &rhs.get()).to_value()
	}

	pub fn checked_mul(self, rhs: Self) -> Value {
		if let Some(integer) = self.n.checked_mul(rhs.n) {
			// SAFETY: It's impossible to multiply even `i64` and get a divisible-by-four one.
			return unsafe { Self::new_raw(integer >> 1) }.to_value();
		}

		(self.to_bigint() * &rhs.get()).to_value()
	}

	pub fn checked_div(self, rhs: Self) -> Result<Value> {
		if let Some(integer) = self.n.checked_div(rhs.n) {
			// It's possible to get an odd number via division, so we have to `new_truncate`.
			return Ok(Self::new_truncate(integer).to_value());
		}

		if rhs == 0 {
			return Err(ErrorKind::DivisionByZero("division").into());
		}

		Ok((self.to_bigint() / &rhs.get()).to_value())
	}

	// technically this is remainder right now...
	pub fn checked_mod(self, rhs: Self) -> Result<Value> {
		if let Some(integer) = self.n.checked_rem(rhs.n) {
			// SAFETY: It's impossible to modulo even `i64` and get an odd one.
			return Ok(unsafe { Self::new_raw(integer) }.to_value());
		}

		if rhs == 0 {
			return Err(ErrorKind::DivisionByZero("modulo").into());
		}

		Ok((self.to_bigint() % &rhs.get()).to_value())
	}

	pub fn checked_pow(self, rhs: Self) -> Value {
		// TODO!
		self.get().pow(rhs.get().try_into().expect("todo: exception for not valid number")).to_value()
	}

	pub fn checked_shl(self, rhs: Self) -> Result<Value> {
		let _ = rhs;
		todo!();
	}

	pub fn checked_shr(self, rhs: Self) -> Result<Value> {
		let _ = rhs;
		todo!();
	}

	pub fn checked_bitand(self, rhs: Self) -> Value {
		let _ = rhs;
		todo!();
	}

	pub fn checked_bitor(self, rhs: Self) -> Value {
		let _ = rhs;
		todo!();
	}

	pub fn checked_bitxor(self, rhs: Self) -> Value {
		let _ = rhs;
		todo!();
	}

	pub fn checked_bitneg(self) -> Value {
		todo!();
	}

	pub fn to_bigint(self) -> BigInt {
		BigInt::from(self.get())
	}
}

pub mod funcs {
	use super::*;

	pub fn op_add(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_add(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_sub(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_sub(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_mul(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_mul(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_div(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		int.checked_div(args[0].try_downcast::<Integer>()?)
	}

	// TODO: verify it's actually modulus
	pub fn op_mod(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		int.checked_mod(args[0].try_downcast::<Integer>()?)
	}

	pub fn op_pow(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		if let Some(float) = args[0].downcast::<Float>() {
			return Ok(((int.get() as Float).powf(float)).to_value());
		}

		Ok(int.checked_pow(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_lth(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int < args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_leq(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int <= args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_gth(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int > args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_geq(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok((int >= args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_cmp(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.cmp(&args[0].try_downcast::<Integer>()?).to_value())
	}

	pub fn op_neg(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(int.checked_neg())
	}

	pub fn op_shl(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		int.checked_shl(args[0].try_downcast::<Integer>()?)
	}

	pub fn op_shr(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		int.checked_shr(args[0].try_downcast::<Integer>()?)
	}

	pub fn op_bitand(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_bitand(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_bitor(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_bitor(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_bitxor(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		Ok(int.checked_bitxor(args[0].try_downcast::<Integer>()?))
	}

	pub fn op_bitneg(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok(int.checked_bitneg())
	}

	pub fn to_text(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Gc<Text>>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn to_float(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Float>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn to_int(int: Integer, args: Args<'_>) -> Result<Value> {
		ConvertTo::<Integer>::convert(&int, args).map(ToValue::to_value)
	}

	pub fn dbg(int: Integer, args: Args<'_>) -> Result<Value> {
		to_text(int, args)
	}

	// pub fn dbg(val: Value, args: Args<'_>) -> Result<Value> {
	// 	if let Some(int) = val.downcast::<Integer>() {
	// 		dbg_int(int, args)
	// 	} else if val.is_identical(Integer::parent()) {
	// 		args.assert_no_arguments()?;
	// 		Ok(Text::from_string(format!("")).to_value())
	// 	}
	// }

	// TODO: in the future, return an enumerable
	pub fn upto(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let max = args[0].try_downcast::<Integer>()?.get();
		let int = int.get();

		if max < int {
			return Ok(List::new().to_value());
		}

		let mut list = List::with_capacity((max - int) as usize);

		for i in int..=max {
			list.push(i.to_value());
		}

		Ok(list.to_value())
	}

	pub fn downto(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_keyword()?;
		args.assert_positional_len(1)?;

		let min = args[0].try_downcast::<Integer>()?.get();
		let int = int.get();

		if min > int {
			return Ok(List::new().to_value());
		}

		let mut list = List::with_capacity((int - min) as usize);

		for i in (min..=int).rev() {
			list.push(i.to_value());
		}

		Ok(list.to_value())
	}

	pub fn times(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		upto(Integer::ZERO, Args::new(&[(int.get() - 1).to_value()], &[]))
	}

	pub fn chr(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;
		let int = int.get();

		if let Some(chr) = u32::try_from(int).ok().and_then(char::from_u32) {
			let mut builder = Text::simple_builder();
			builder.push(chr);
			Ok(builder.finish().to_value())
		} else {
			Err(format!("oops, number {int:x} is out of bounds!").into())
		}
	}

	pub fn is_even(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int.n & 0b10 == 0b00).to_value())
	}

	pub fn is_odd(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int.n & 0b10 == 0b10).to_value())
	}

	pub fn is_zero(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int == Integer::ZERO).to_value())
	}

	pub fn is_one(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int == Integer::ONE).to_value())
	}

	pub fn is_positive(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int.n > 0).to_value())
	}

	pub fn is_negative(int: Integer, args: Args<'_>) -> Result<Value> {
		args.assert_no_arguments()?;

		Ok((int.n < 0).to_value())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IntegerClass;

impl Singleton for IntegerClass {
	fn instance() -> crate::Value {
		use once_cell::sync::OnceCell;

		static INSTANCE: OnceCell<crate::Value> = OnceCell::new();

		*INSTANCE.get_or_init(|| {
			create_class! { "Integer", parent Object::instance();
				Intern::op_neg => method funcs::op_neg,
				Intern::op_add => function Integer::qs_op_add,
				// Intern::op_add => method funcs::op_add,
				Intern::op_sub => method funcs::op_sub,
				Intern::op_mul => method funcs::op_mul,
				Intern::op_div => method funcs::op_div,
				Intern::op_mod => method funcs::op_mod,
				Intern::op_pow => method funcs::op_pow,

				Intern::op_lth => method funcs::op_lth,
				Intern::op_leq => method funcs::op_leq,
				Intern::op_gth => method funcs::op_gth,
				Intern::op_geq => method funcs::op_geq,
				Intern::op_cmp => method funcs::op_cmp,

				Intern::op_shl => method funcs::op_shl,
				Intern::op_shr => method funcs::op_shr,
				Intern::op_bitand => method funcs::op_bitand,
				Intern::op_bitor => method funcs::op_bitor,
				Intern::op_bitxor => method funcs::op_bitxor,
				Intern::op_bitneg => method funcs::op_bitneg,

				Intern::times => method funcs::times,
				Intern::upto => method funcs::upto,
				Intern::downto => method funcs::downto,

				Intern::is_even => method funcs::is_even,
				Intern::is_odd => method funcs::is_odd,
				Intern::is_zero => method funcs::is_zero,
				Intern::is_positive => method funcs::is_positive,
				Intern::is_negative => method funcs::is_negative,

				Intern::chr => method funcs::chr,
				Intern::to_text => method funcs::to_text,
				Intern::to_float => method funcs::to_float,
				Intern::to_int => method funcs::to_int,
				Intern::dbg => method funcs::dbg,
			}
		})
	}
}

impl InstanceOf for Integer {
	type Parent = IntegerClass;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::*;
	use crate::ToValue;

	#[test]
	fn test_is_a() {
		assert!(Integer::is_a(0i64.to_value()));
		assert!(Integer::is_a(1i64.to_value()));
		assert!(Integer::is_a((-123i64).to_value()));
		assert!(Integer::is_a(14i64.to_value()));
		assert!(Integer::is_a((-1i64).to_value()));
		assert!(Integer::is_a(Integer::MIN.to_value()));
		assert!(Integer::is_a(Integer::MAX.to_value()));

		assert!(!Integer::is_a(Value::TRUE.to_value()));
		assert!(!Integer::is_a(Value::FALSE.to_value()));
		assert!(!Boolean::is_a(Value::NULL.to_value()));
		assert!(!Integer::is_a(1.0.to_value()));
		assert!(!Integer::is_a("hello".to_value()));
		assert!(!Integer::is_a(RustFn::NOOP.to_value()));
	}

	#[test]
	fn test_get() {
		assert_eq!(Integer::ZERO, <Integer as Convertible>::get(Integer::ZERO.into()));
		assert_eq!(Integer::ONE, <Integer as Convertible>::get(Integer::ONE.into()));
		assert_eq!(
			Integer::new_truncate(-123),
			<Integer as Convertible>::get(Integer::new_truncate(-123i64).into())
		);
		assert_eq!(
			Integer::new_truncate(14),
			<Integer as Convertible>::get(Integer::new_truncate(14i64).into())
		);
		assert_eq!(
			Integer::new_truncate(-1),
			<Integer as Convertible>::get(Integer::new_truncate(-1i64).into())
		);
		assert_eq!(Integer::MIN, <Integer as Convertible>::get(Integer::MIN.into()));
		assert_eq!(Integer::MAX, <Integer as Convertible>::get(Integer::MAX.into()));
	}

	#[test]
	fn test_convert_to_float() {
		// TODO: how do we want to test conversions
	}

	#[test]
	fn test_convert_to_text_noargs() {
		macro_rules! to_text {
			($num:expr) => {
				ConvertTo::<Gc<Text>>::convert(&$num, Args::default())
					.unwrap()
					.as_ref()
					.unwrap()
					.as_str()
			};
		}

		assert_eq!("0", to_text!(Integer::ZERO));
		assert_eq!("1", to_text!(Integer::ONE));
		assert_eq!("-123", to_text!(Integer::new_truncate(-123)));
		assert_eq!("14", to_text!(Integer::new_truncate(14)));
		assert_eq!("-1", to_text!(Integer::new_truncate(-1)));
		assert_eq!(Integer::MIN.to_string(), to_text!(Integer::MIN));
		assert_eq!(Integer::MAX.to_string(), to_text!(Integer::MAX));
	}

	#[test]
	fn test_convert_to_text_bad_args_error() {
		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ZERO,
			Args::new(&[Value::TRUE.to_value()], &[])
		)
		.is_err());
		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ZERO,
			Args::new(&[], &[("A", Value::TRUE.to_value())])
		)
		.is_err());
		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ZERO,
			Args::new(&[Value::TRUE.to_value()], &[("A", Value::TRUE.to_value())])
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ZERO,
			Args::new(
				&[Value::TRUE.to_value()],
				&[("base", Value::from(Integer::new_truncate(2)).to_value())]
			)
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ZERO,
			Args::new(
				&[Value::TRUE.to_value()],
				&[
					("base", Value::from(Integer::new_truncate(2)).to_value()),
					("A", Value::TRUE.to_value())
				]
			)
		)
		.is_err());

		assert!(ConvertTo::<Gc<Text>>::convert(
			&Integer::ONE,
			Args::new(
				&[],
				&[
					("base", Value::from(Integer::new_truncate(2)).to_value()),
					("A", Value::TRUE.to_value())
				]
			)
		)
		.is_err());
	}

	#[test]
	#[ignore]
	fn test_convert_to_text_different_radix() {
		macro_rules! to_text {
			($num:expr, $radix:expr) => {
				ConvertTo::<Gc<Text>>::convert(
					&$num,
					Args::new(
						&[],
						&[("base", Value::from(Integer::new_truncate($radix as Inner)).to_value())],
					),
				)
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
			};
		}

		for radix in 2..=36 {
			assert_eq!(
				radix_fmt::radix(0 as Inner, radix).to_string(),
				to_text!(Integer::ZERO, radix)
			);
			assert_eq!(radix_fmt::radix(1 as Inner, radix).to_string(), to_text!(Integer::ONE, radix));
			assert_eq!(
				radix_fmt::radix(-123 as Inner, radix).to_string(),
				to_text!(Integer::new_truncate(-123), radix)
			);
			assert_eq!(
				radix_fmt::radix(14 as Inner, radix).to_string(),
				to_text!(Integer::new_truncate(14), radix)
			);
			assert_eq!(
				radix_fmt::radix(-1 as Inner, radix).to_string(),
				to_text!(Integer::NEG_ONE, radix)
			);
			assert_eq!(
				radix_fmt::radix(Integer::MIN.get(), radix).to_string(),
				to_text!(Integer::MIN, radix)
			);
			assert_eq!(
				radix_fmt::radix(Integer::MAX.get(), radix).to_string(),
				to_text!(Integer::MAX, radix)
			);
		}
	}

	#[test]
	fn op_neg() {
		// println!("{} {} {} {}", -i8::Integer::MAX, i8::Integer::MAX, i8::Integer::MIN, 0);
		// println!("{Integer::MAX} {:?}", (-Integer::MAX).to_value().downcast::<Integer>().unwrap());
		// println!("{Integer::MIN} {:?}", (-Integer::MIN).to_value().downcast::<Integer>().unwrap());

		assert_code! {
			r#"
				# Make sure we aren't using a `-` prefix when parsing
				assert( -(1) == (0 - 1) );
				assert( -(0) == 0 );
				assert( -(-(1)) == 1 );
				#assert( -({{MAX}}) == {{MIN}} );
				#assert( -({{MIN}}) == {{MAX}} );
			"#,
		}
	}

	/*
				create_class! { "Integer", parent Object::instance();
					Intern::op_neg => method funcs::op_neg,
					Intern::op_add => method funcs::op_add,
					Intern::op_sub => method funcs::op_sub,
					Intern::op_mul => method funcs::op_mul,
					Intern::op_div => method funcs::op_div,
					Intern::op_mod => method funcs::op_mod,
					Intern::op_pow => method funcs::op_pow,

					Intern::op_lth => method funcs::op_lth,
					Intern::op_leq => method funcs::op_leq,
					Intern::op_gth => method funcs::op_gth,
					Intern::op_geq => method funcs::op_geq,
					Intern::op_cmp => method funcs::op_cmp,

					Intern::op_shl => method funcs::op_shl,
					Intern::op_shr => method funcs::op_shr,
					Intern::op_bitand => method funcs::op_bitand,
					Intern::op_bitor => method funcs::op_bitor,
					Intern::op_bitxor => method funcs::op_bitxor,
					Intern::op_bitneg => method funcs::op_bitneg,

					Intern::times => method funcs::times,
					Intern::upto => method funcs::upto,
					Intern::downto => method funcs::downto,

					Intern::is_even => method funcs::is_even,
					Intern::is_odd => method funcs::is_odd,
					Intern::is_zero => method funcs::is_zero,
					Intern::is_positive => method funcs::is_positive,
					Intern::is_negative => method funcs::is_negative,

					Intern::chr => method funcs::chr,
					Intern::to_text => method funcs::to_text,
					Intern::to_float => method funcs::to_float,
					Intern::to_int => method funcs::to_int,
					Intern::dbg => method funcs::dbg,
				}
			})
	*/
}
