use crate::value::ty::{text, Text};
use crate::value::Gc;
use crate::{ToValue, Value};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

const TAG: u64 = 0b100_0100;

const fn offset(x: u64) -> u64 {
	(x << 7) | TAG
}

#[rustfmt::skip]
macro_rules! variant_name {
	($name:ident) => (stringify!($name));
	($_name:ident $value:literal) => ($value);
}

macro_rules! define_interned {
	($($name:ident $($value:literal)?)*) => {
		/// Strings that are statically present within Quest.
		///
		/// Since these strings are known ahead of time, and are usually frequently used, interning
		/// them allows for extremely fast lookups and comparisons.
		#[derive(Clone, Copy)]
		#[repr(transparent)]
		pub struct Intern(u64);

		#[allow(non_upper_case_globals)]
		impl Intern {
			$(
				#[doc = concat!("Represents the `\"", variant_name!($name $($value)?), "\"` string in Quest")]
				pub const $name: Self = Self(offset(__InternHelper::$name as _));
			)*
		}

		#[allow(non_camel_case_types)]
		enum __InternHelper {
			$($name,)* __LAST
		}

		const INTERN_LENGTH: usize = __InternHelper::__LAST as usize;

		impl Intern {
			/// Converts `self` to its string representation.
			#[inline]
			#[must_use]
			pub const fn as_str(self) -> &'static str {
				const STRINGS: [&'static str; INTERN_LENGTH] = [ $(variant_name!($name $($value)?)),* ];

				STRINGS[self.as_index()]
			}

			/// Gets the [fast hash](crate::value::ty::text::fast_hash) corresponding to `self`'s
			/// [string representation](Self::as_str).
			#[must_use]
			pub const fn fast_hash(self) -> u64 {
				const HASHES: [u64; INTERN_LENGTH] = [ $(text::fast_hash(Intern::$name.as_str())),* ];

				HASHES[self.as_index()]
			}
		}


		impl TryFrom<&'_ Text> for Intern {
			type Error = ();

			/// Attempts to convert the `text` into its corresponding [`Intern`] representation.
			#[allow(non_upper_case_globals)]
			fn try_from(text: &Text) -> Result<Self, Self::Error> {
				$(
					const $name: u8 = Intern::$name.fast_hash() as u8;
				)*

				match text.fast_hash() as u8 {
					$($name if *text == Intern::$name => Ok(Self::$name),)*
					_ => Err(())
				}
			}
		}
	};
}

define_interned! {
	// underscore methods
	__parents__ __id__ __name__
	__get_attr__ __get_unbound_attr__ __set_attr__
	__del_attr__ __has_attr__ __call_attr__

	// Constants
	r#true "true" r#false "false" null

	// Classes
	Boolean BoundFn Callable Class Float Integer Kernel List
	Null Object Pristine RustFn Scope Text
	Frame Block

	// Operators
	op_add "+" op_sub "-" op_mul "*" op_div "/" op_mod "%" op_pow "**"
	op_eql "==" op_neq "!=" op_lth "<" op_leq "<=" op_gth ">" op_geq ">=" op_cmp "<=>"
	op_not "!" op_neg "-@" op_index "[]" op_index_assign "[]=" op_call "()" op_assign "="
	op_shl "<<" op_shr ">>" op_bitand "&" op_bitor "|" op_bitxor "^" op_bitneg

	// Conversions
	dbg to_text to_num to_bool to_list to_int to_float

	// `Object` functions
	hash clone itself
	tap pipe then and_then r#else "else" or_else or and
	display freeze

	// Kernel functions
	if_cascade ifl r#if "if"
	r#while "while" r#return "return"
	exit abort assert object print
	spawn dump // both are temporary

	// Frame and Block Functions
	resume restart create_frame

	// String functions
	join concat len

	// List functions
	push pop shift unshift

	// Enumerator functions
	map filter reduce each next sum

	// Integer functions
	upto downto times chr
	is_even is_odd is_zero is_positive is_negative

	// Float functions
	is_whole

}

// Note that this has to be implemented like this because we manually implement `Hash`.
impl Eq for Intern {}
impl PartialEq for Intern {
	fn eq(&self, rhs: &Self) -> bool {
		self.bits() == rhs.bits()
	}
}

impl Hash for Intern {
	fn hash<H: Hasher>(&self, h: &mut H) {
		h.write_u64(self.fast_hash());
	}
}

impl Intern {
	pub unsafe fn from_bits(bits: u64) -> Self {
		Self(bits)
	}

	pub const fn bits(self) -> u64 {
		self.0
	}

	const fn as_index(self) -> usize {
		(self.bits() >> 7) as usize
	}

	pub(crate) const fn try_from_repr(repr: u64) -> Option<Self> {
		if repr & 0b111_1111 == TAG {
			debug_assert!(repr <= offset(INTERN_LENGTH as u64));

			Some(unsafe { std::mem::transmute::<u64, Self>(repr) })
		} else {
			None
		}
	}

	/// Converts `self` to its `Text` representation.
	#[must_use]
	pub fn as_text(self) -> Gc<Text> {
		use once_cell::sync::OnceCell;

		// We only need the const for the `TEXTS` initializer
		#[allow(clippy::declare_interior_mutable_const)]
		const BLANK_TEXT: OnceCell<Gc<Text>> = OnceCell::new();

		static TEXTS: [OnceCell<Gc<Text>>; INTERN_LENGTH] = [BLANK_TEXT; INTERN_LENGTH];

		*TEXTS[self.as_index()].get_or_init(|| {
			let text = Text::from_static_str(self.as_str());
			text.as_ref().unwrap().freeze();
			text
		})
	}
}

impl From<Intern> for Value<Gc<Text>> {
	fn from(intern: Intern) -> Self {
		intern.as_text().into()
	}
}

impl ToValue for Intern {
	fn to_value(self) -> Value {
		Value::from(self).to_value()
	}
}

impl Debug for Intern {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_tuple("Intern").field(&self.as_str()).finish()
		} else {
			f.write_str(self.as_str())
		}
	}
}

impl Display for Intern {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str(self.as_str())
	}
}
