use crate::value::ty::{text, Text};
use crate::value::Gc;
use crate::{ToValue, Value};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

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

		const BUILTIN_LENGTH: usize = __InternHelper::__LAST as usize;

		impl Intern {
			pub const fn as_str_const(self) -> &'static str {
				const STRINGS: [&'static str; BUILTIN_LENGTH] = [
					$(variant_name!($name $($value)?)),*
				];

				debug_assert!(self.is_builtin());
				STRINGS[self.as_index()]
			}

			/// Converts `self` to its string representation.
			#[inline]
			#[must_use]
			pub fn as_str(self) -> &'static str {
				if self.is_builtin() {
					self.as_str_const()
				} else {
					let text = intern_to_text().read().unwrap()[self.as_index() - BUILTIN_LENGTH];
					unsafe {
						std::mem::transmute::<&str, &'static str>(text.as_ref().unwrap().as_str())
					}
				}
			}

			const fn fast_hash_builtin(self) -> u64 {
				const HASHES: [u64; BUILTIN_LENGTH] = [ $(text::fast_hash(Intern::$name.as_str_const())),* ];

				debug_assert!(self.is_builtin());
				HASHES[self.as_index()]
			}

			/// Gets the [fast hash](crate::value::ty::text::fast_hash) corresponding to `self`'s
			/// [string representation](Self::as_str).
			#[must_use]
			pub fn fast_hash(self) -> u64 {
				if self.is_builtin() {
					self.fast_hash_builtin()
				} else {
					intern_to_text().read().unwrap()[self.as_index() - BUILTIN_LENGTH].as_ref().unwrap().fast_hash()
				}
			}
		}


		impl TryFrom<&'_ Text> for Intern {
			type Error = ();

			/// Attempts to convert the `text` into its corresponding [`Intern`] representation.
			#[allow(non_upper_case_globals)]
			fn try_from(text: &Text) -> Result<Self, Self::Error> {
				$(
					const $name: u8 = Intern::$name.fast_hash_builtin() as u8;
				)*

				match text.fast_hash() as u8 {
					$($name if *text == Intern::$name => Ok(Self::$name),)*
					_ => text_to_intern().get(text.as_ref()).map(|x| *x.value()).ok_or(())
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
	display freeze dup

	// Kernel functions
	if_cascade ifl r#if "if"
	r#while "while" r#return "return"
	exit abort assert object print rand
	spawn dump // both are temporary

	// Frame and Block Functions
	resume restart create_frame __block__ __args__

	// String functions
	join concat len

	// List functions
	push pop shift unshift product shuffle is_empty

	// Enumerator functions
	map filter reduce each next sum iter tap_each count is_any are_all

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

fn intern_to_text() -> &'static RwLock<Vec<Gc<Text>>> {
	static INTERN_TO_TEXT: OnceCell<RwLock<Vec<Gc<Text>>>> = OnceCell::new();

	INTERN_TO_TEXT.get_or_init(Default::default)
}

fn text_to_intern() -> &'static DashMap<&'static str, Intern> {
	static TEXT_TO_INTERN: OnceCell<DashMap<&'static str, Intern>> = OnceCell::new();

	TEXT_TO_INTERN.get_or_init(Default::default)
}

impl Intern {
	pub fn new(text: Gc<Text>) -> crate::Result<Self> {
		let textref = text.as_ref()?;

		if let Ok(intern) = Self::try_from(&*textref) {
			return Ok(intern);
		}

		textref.freeze();
		text.do_not_free();

		// SAFETY: Since `text` will never be freed, and is frozen, we know that it's contents will
		// always be the same value for the remainder of the program. Thus, we can extend its lifetime.
		let string = unsafe { std::mem::transmute::<&str, &'static str>(textref.as_str()) };
		let intern = Self(offset((BUILTIN_LENGTH + text_to_intern().len()) as u64));

		text_to_intern().insert(string, intern);
		intern_to_text().write().unwrap().push(text);

		Ok(intern)
	}

	pub unsafe fn from_bits_unchecked(bits: u64) -> Self {
		Self(bits)
	}

	const fn is_builtin(self) -> bool {
		self.as_index() < BUILTIN_LENGTH
	}

	pub const fn bits(self) -> u64 {
		self.0
	}

	const fn as_index(self) -> usize {
		(self.bits() >> 7) as usize
	}

	pub(crate) unsafe fn try_from_repr(bits: u64) -> Option<Self> {
		if bits & 0b111_1111 == TAG {
			debug_assert!((bits as usize >> 7) < (BUILTIN_LENGTH + text_to_intern().len()));
			Some(Self::from_bits_unchecked(bits))
		} else {
			None
		}
	}

	/// Converts `self` to its `Text` representation.
	#[must_use]
	pub fn as_text(self) -> Gc<Text> {
		// We only need the const for the `TEXTS` initializer
		#[allow(clippy::declare_interior_mutable_const)]
		const BLANK_TEXT: OnceCell<Gc<Text>> = OnceCell::new();
		static TEXTS: [OnceCell<Gc<Text>>; BUILTIN_LENGTH] = [BLANK_TEXT; BUILTIN_LENGTH];

		if !self.is_builtin() {
			return intern_to_text().read().unwrap()[self.as_index() - BUILTIN_LENGTH];
		}

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
