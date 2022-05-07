use crate::value::ty::Text;
use crate::value::{AsAny, Gc};
use crate::{AnyValue, Value};
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

const TAG: u64 = 0b100_0100;

const fn offset(x: u64) -> u64 {
	(x << 7) | TAG
}

macro_rules! define_interned {
	(@ $name:ident) => (stringify!($name));
	(@ $_name:ident $value:literal) => ($value);

	($($name:ident $($value:literal)?)*) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[allow(non_camel_case_types)]
		#[repr(u64)]
		#[non_exhaustive]
		pub enum Intern {
			$($name = offset(__InternHelper::$name as _),)*
			#[doc(hidden)]
			__LAST = offset(__InternHelper::__LAST as _),
		}

		#[allow(non_camel_case_types)]
		enum __InternHelper {
			$($name,)* __LAST
		}

		impl Intern {
			pub const fn as_str(self) -> &'static str {
				const STRINGS: [&'static str; Intern::__LAST.as_index()] = [
					$(define_interned!(@ $name $($value)?)),*
				];

				STRINGS[self.as_index()]
			}

			pub fn from_str(s: &str) -> Option<Self> {
				match s {
					$(define_interned!(@ $name $($value)?) => Some(Self::$name),)*
					_ => None
				}
			}

			pub const fn fast_hash(self) -> u64 {
				const HASHES: [u64; Intern::__LAST.as_index()] = [
					$(crate::value::ty::text::fast_hash(define_interned!(@ $name $($value)?)), )*
				];

				HASHES[self.as_index()]
			}

		}
	};
}

define_interned! {
	__parents__
	__id__
	__get_attr__ __get_unbound_attr__ __set_attr__
	__del_attr__ __has_attr__ __call_attr__

	concat
	len

	op_add "+" op_sub "-" op_mul "*" op_div "/" op_mod "%" op_pow "**"
	op_eql "==" op_neq "!=" op_lth "<" op_leq "<=" op_gth ">" op_geq ">=" op_cmp "<=>"
	op_not "!" op_neg "-@" op_index "[]" op_index_assign "[]=" op_call "()" op_assign "="

	at_text "@text" at_num "@num" at_bool "@bool" at_list "@list"
	at_int "@int" at_float "@float"

	hash
	clone
	tap tap_into then and_then or_else or and itself

	r#if "if"
	r#while "while"
	r#else "else"
	r#return "return"

	inspect print freeze
}

impl Hash for Intern {
	fn hash<H: Hasher>(&self, h: &mut H) {
		h.write_u64(self.fast_hash());
	}
}

impl Intern {
	const fn as_index(self) -> usize {
		((self as u64) >> 7) as usize
	}

	pub(crate) const fn try_from_repr(repr: u64) -> Option<Self> {
		if repr & 0b111_1111 == TAG {
			debug_assert!(repr <= Self::__LAST as u64);

			Some(unsafe { std::mem::transmute::<u64, Self>(repr) })
		} else {
			None
		}
	}

	pub fn as_text(self) -> Gc<Text> {
		use once_cell::sync::OnceCell;

		const AMNT: usize = Intern::__LAST.as_index() - 1;
		const BLANK_TEXT: OnceCell<Gc<Text>> = OnceCell::new();

		static TEXTS: [OnceCell<Gc<Text>>; AMNT] = [BLANK_TEXT; AMNT];

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

impl AsAny for Intern {
	fn as_any(self) -> AnyValue {
		Value::from(self).any()
	}
}

impl Display for Intern {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl Deref for Intern {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}
