use std::fmt::{self, Binary, Debug, Formatter};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct Flags(AtomicU32);

impl Flags {
	pub const USER0: u32 = 1 << 0;
	pub const USER1: u32 = 1 << 1;
	pub const USER2: u32 = 1 << 2;
	pub const USER3: u32 = 1 << 3;
	pub const USER4: u32 = 1 << 4;
	pub const USER5: u32 = 1 << 5;
	pub const USER6: u32 = 1 << 6;
	pub const USER7: u32 = 1 << 7;
	pub const USER8: u32 = 1 << 8;
	pub const USER9: u32 = 1 << 9;
	pub const USER10: u32 = 1 << 10;
	pub const USER11: u32 = 1 << 11;
	pub const USER12: u32 = 1 << 12;
	pub const USER13: u32 = 1 << 13;
	pub const USER14: u32 = 1 << 14;
	pub const USER15: u32 = 1 << 15;
	pub const USER_FLAGS_MASK: u32 = (Self::USER15 << 1) - 1;

	pub(crate) const FROZEN: u32 = 1 << 16;
	pub(crate) const NOFREE: u32 = 1 << 17;
	pub(crate) const GCMARK: u32 = 1 << 18;
	pub(crate) const ATTR_MAP: u32 = 1 << 19;
	pub(crate) const MULTI_PARENT: u32 = 1 << 20;
	pub(crate) const LOCK_PARENTS: u32 = 1 << 21;
	pub(crate) const LOCK_ATTRIBUTES: u32 = 1 << 22;
	const UNUSED8: u32 = 1 << 23;
	const UNUSED7: u32 = 1 << 24;
	const UNUSED6: u32 = 1 << 25;
	const UNUSED5: u32 = 1 << 26;
	const UNUSED4: u32 = 1 << 27;
	const UNUSED3: u32 = 1 << 28;
	const UNUSED2: u32 = 1 << 29;
	const UNUSED1: u32 = 1 << 30;
	const UNUSED0: u32 = 1 << 31;

	pub const fn new(flags: u32) -> Self {
		Self(AtomicU32::new(flags))
	}

	#[inline]
	pub fn insert_user(&self, flag: u32) {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		self
			.0
			.fetch_or(flag & Self::USER_FLAGS_MASK, Ordering::SeqCst);
	}

	// Attempts to acquire a "lock" on a flag mask, such that all the flags are valid
	// Returns `true` if we could acquire it.
	pub fn try_acquire_all_user(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		self
			.0
			.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
				if (value & (flag & Self::USER_FLAGS_MASK)) == 0 {
					Some(value | (flag & &Self::USER_FLAGS_MASK))
				} else {
					None
				}
			})
			.is_ok()
	}

	#[inline]
	pub(crate) fn insert_internal(&self, flag: u32) {
		debug_assert_eq!(flag & Self::USER_FLAGS_MASK, 0, "attempted to set user flags.");

		self
			.0
			.fetch_or(flag & !Self::USER_FLAGS_MASK, Ordering::SeqCst);
	}

	// Attempts to acquire a "lock" on a flag mask, such that all the flags are valid
	// Returns `true` if we could acquire it.
	pub(crate) fn try_acquire_all_internal(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & Self::USER_FLAGS_MASK, 0, "attempted to set user flags.");

		self
			.0
			.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
				if (value & (flag & !Self::USER_FLAGS_MASK)) == 0 {
					Some(value | (flag & !Self::USER_FLAGS_MASK))
				} else {
					None
				}
			})
			.is_ok()
	}

	pub fn get(&self) -> u32 {
		self.0.load(Ordering::SeqCst)
	}

	pub fn mask(&self, mask: u32) -> u32 {
		self.get() & mask
	}

	pub fn contains(&self, flag: u32) -> bool {
		self.mask(flag) == flag
	}

	pub fn contains_any(&self, flag: u32) -> bool {
		self.mask(flag) != 0
	}

	#[inline]
	pub fn remove_user(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		// TODO: is this the right way to remove
		self
			.0
			.fetch_and(!(flag & Self::USER_FLAGS_MASK), Ordering::SeqCst)
			& flag != 0
	}

	pub(crate) fn remove_internal(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & Self::USER_FLAGS_MASK, 0, "attempted to set user flags.");

		// FIXME: bitwise flag with user flags mask, but is it right?
		self
			.0
			.fetch_and(!(flag & !Self::USER_FLAGS_MASK), Ordering::SeqCst)
			& flag != 0
	}
}

impl Debug for Flags {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Flags(")?;
		let mut is_first = true;

		macro_rules! check {
			($($flag:ident)*) => {
				$(
					if self.contains(Self::$flag) {
						if is_first {
							is_first = false;
						} else {
							write!(f, " | ")?;
						}
						write!(f, stringify!($flag))?;
					}
				)*
			};
		}

		check!(
			USER0 USER1 USER2 USER3 USER4 USER5 USER6 USER7 USER8 USER9
			USER10 USER11 USER12 USER13 USER14 USER15
			FROZEN NOFREE GCMARK ATTR_MAP MULTI_PARENT
			LOCK_PARENTS LOCK_ATTRIBUTES
			UNUSED8 UNUSED7 UNUSED6 UNUSED5 UNUSED4 UNUSED3 UNUSED2 UNUSED1 UNUSED0
		);

		let _ = is_first;

		write!(f, ")")
	}
}

impl Binary for Flags {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Flags({:032b})", self.get())
	}
}

impl Clone for Flags {
	fn clone(&self) -> Self {
		Self::new(self.get())
	}
}