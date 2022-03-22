use crate::value::{Gc, AsAny};
use crate::value::ty::Text;
use crate::{Value, AnyValue};
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

macro_rules! define_interned {
	(@ $name:ident) => (stringify!($name));
	(@ $_name:ident $value:literal) => ($value);

	($first:ident $($name:ident $($value:literal)?)*) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		#[allow(non_camel_case_types)]
		#[repr(u64)]
		#[non_exhaustive]
		pub enum Intern {
			$first = 1,
			$($name,)*
			#[doc(hidden)]
			__LAST
		}

		impl Intern {
			pub const fn as_str(self) -> &'static str {
				match self {
					Self::$first => &define_interned!(@ $first),
					$(Self::$name => &define_interned!(@ $name $($value)?),)*
					Self::__LAST => panic!("don't use `__LAST`"),
				}
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
	op_eql "=="
	op_neq "!="
	op_not "!"
	op_call "()"
	op_add "+"

	at_text "@text"
	at_num "@num"
	at_bool "@bool"

	hash
	clone
	tap tap_into then and_then or_else or and itself

	r#if "if"
	r#while "while"
	r#else "else"

	inspect
	at_int "@int"
	at_float "@float"
	at_list "@list"
}

impl Deref for Intern {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Intern {
	pub fn as_text(self) -> Gc<Text> {
		use once_cell::sync::OnceCell;

		const AMNT: usize = Intern::__LAST as usize - 1;
		const BLANK_TEXT: OnceCell<Gc<Text>> = OnceCell::new();

		static TEXTS: [OnceCell<Gc<Text>>; AMNT] = [BLANK_TEXT; AMNT];

		*TEXTS[self as usize].get_or_init(|| {
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
