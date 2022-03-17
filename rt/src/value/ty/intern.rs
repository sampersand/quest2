use std::ops::Deref;

macro_rules! define_interned {
	(@ $name:ident) => (stringify!($name));
	(@ $_name:ident $value:literal) => ($value);

	($first:ident $($name:ident $($value:literal)?)*) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		#[allow(non_camel_case_types)]
		#[repr(usize)]
		pub enum Intern {
			$first = 1,
			$($name),*
		}

		impl Deref for Intern {
			type Target = &'static str;

			fn deref(&self) -> &Self::Target {
				match *self {
					Self::$first => &define_interned!(@ $first),
					$(Self::$name => &define_interned!(@ $name $($value)?)),*
				}
			}
		}
	};
}


define_interned! {
	clone
	at_text "@text"
	at_num "@num" 
}

impl From<Intern> for crate::Value<crate::value::gc::Gc<crate::value::ty::Text>> {
	fn from(intern: Intern) -> Self {
		crate::value::ty::Text::from_static_str(&intern).into()
	}
}

impl crate::value::AsAny for Intern {
	fn as_any(self) -> crate::AnyValue {
		crate::Value::from(self).any()
	}
}
