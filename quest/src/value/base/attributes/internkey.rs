use crate::value::Intern;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct InternKey(u64);

const FROZEN_BIT: u64 = 0b10_0000;

impl InternKey {
	#[allow(unused)]
	pub const fn new(intern: Intern) -> Self {
		Self(intern as u64)
	}

	#[allow(unused)]
	pub const fn new_frozen(intern: Intern) -> Self {
		Self((intern as u64) | FROZEN_BIT)
	}

	pub const fn is_frozen(self) -> bool {
		self.0 & FROZEN_BIT != 0
	}

	pub const fn try_from_repr(repr: u64) -> Option<Self> {
		if Intern::try_from_repr(repr & !FROZEN_BIT).is_some() {
			Some(Self(repr))
		} else {
			None
		}
	}

	pub const fn as_intern(self) -> Intern {
		unsafe { std::mem::transmute::<u64, Intern>(self.0 & !FROZEN_BIT) }
	}
}

impl PartialEq for InternKey {
	fn eq(&self, rhs: &Self) -> bool {
		self.as_intern() == rhs.as_intern()
	}
}

impl Hash for InternKey {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.0.hash(h);
	}
}
