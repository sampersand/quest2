use std::fmt::{self, Binary, Debug, Formatter};
use std::sync::atomic::{AtomicU32, Ordering};

mod typeflag;
pub use typeflag::{HasTypeFlag, TypeFlag};

/// Flags corresponding to a [`Header`](crate::value::base::Header).
///
/// [`Flags`] is split up into two parts: internal flags and user-definable flags. User-definable
/// flags are freely usable, and can be utilized in any way you want. (For example, the [`Text`](
/// crate::value::ty::Text) type uses the flags to represent embeddedness, and even encodes the
/// length of an embedded string in flags.)
///
/// Internal flags shouldn't really be touched by external users, and are used internally by
/// [`Base`](crate::value::base::Base) to keep track of things. (For example, garbage-collection
/// marking, whether the object is frozen, etc.)
#[derive(Default)]
pub struct Flags(AtomicU32); // This entire type needs to be reworked with unsafety rules.

sa::const_assert_eq!(1 << Flags::TYPE_FLAG_BITSHIFT, Flags::TYPE_FLAG1);

impl Flags {
	/// User flag 0.
	pub const USER0: u32 = 1 << 0;
	/// User flag 1.
	pub const USER1: u32 = 1 << 1;
	/// User flag 2.
	pub const USER2: u32 = 1 << 2;
	/// User flag 3.
	pub const USER3: u32 = 1 << 3;
	/// User flag 4.
	pub const USER4: u32 = 1 << 4;
	/// User flag 5.
	pub const USER5: u32 = 1 << 5;
	/// User flag 6.
	pub const USER6: u32 = 1 << 6;
	/// User flag 7.
	pub const USER7: u32 = 1 << 7;
	/// User flag 8.
	pub const USER8: u32 = 1 << 8;
	/// User flag 9.
	pub const USER9: u32 = 1 << 9;
	/// User flag 0.
	pub const USER10: u32 = 1 << 10;
	/// User flag 1.
	pub const USER11: u32 = 1 << 11;
	/// User flag 2.
	pub const USER12: u32 = 1 << 12;
	/// User flag 3.
	pub const USER13: u32 = 1 << 13;
	/// User flag 4.
	pub const USER14: u32 = 1 << 14;
	/// User flag 5.
	pub const USER15: u32 = 1 << 15;
	/// The mask that is applied to user flags.
	pub const USER_FLAGS_MASK: u32 = (Self::USER15 << 1) - 1;

	/// Set if the value is frozen.
	pub(crate) const FROZEN: u32 = 1 << 16;
	/// Set if the value's data shouldn't be freed when garbage collecting.
	pub(crate) const NOFREE: u32 = 1 << 17;
	/// Set when marking via garbage collection
	pub(crate) const GCMARK: u32 = 1 << 18;
	/// Set if attributes is a map, instead of a list.
	pub(crate) const ATTR_MAP: u32 = 1 << 19;
	/// Set if more than one parent is defined on a type.
	pub(crate) const MULTI_PARENT: u32 = 1 << 20;
	const _UNUSED_21: u32 = 1 << 21;
	const _UNUSED_22: u32 = 1 << 22;
	const _UNUSED_23: u32 = 1 << 23;
	const _UNUSED_24: u32 = 1 << 24;
	const _UNUSED_25: u32 = 1 << 25;
	const _UNUSED_26: u32 = 1 << 26;
	const _UNUSED_27: u32 = 1 << 27;

	const TYPE_FLAG_BITSHIFT: u32 = 28;
	const TYPE_FLAG1: u32 = 1 << 28;
	const TYPE_FLAG2: u32 = 1 << 29;
	const TYPE_FLAG3: u32 = 1 << 30;
	const TYPE_FLAG4: u32 = 1 << 31;
	const TYPE_FLAG_MASK: u32 =
		Self::TYPE_FLAG1 | Self::TYPE_FLAG2 | Self::TYPE_FLAG3 | Self::TYPE_FLAG4;

	/// Creates new [`Flags`].
	#[must_use]
	pub const fn new(flags: u32) -> Self {
		Self(AtomicU32::new(flags))
	}

	pub fn type_flag(&self) -> TypeFlag {
		let bits = self.mask(Self::TYPE_FLAG_MASK);

		// SAFETY: We know `bits` are a valid TypeFlag representation, as there's no way to create
		// an invalid one.
		unsafe { TypeFlag::from_bits_unchecked(bits) }
	}

	/// Inserts a user-defined flag.
	#[inline]
	pub fn insert_user(&self, flag: u32) {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		self.0.fetch_or(flag & Self::USER_FLAGS_MASK, Ordering::SeqCst);
	}

	/// Attempts to acquire a "lock" on a flag mask, such that all the flags are valid
	/// Returns `true` if we could acquire it.
	pub fn try_acquire_all_user(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		self
			.0
			.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
				if (value & (flag & Self::USER_FLAGS_MASK)) == 0 {
					Some(value | (flag & Self::USER_FLAGS_MASK))
				} else {
					None
				}
			})
			.is_ok()
	}

	/// inserts an internal flag.
	#[inline]
	pub(crate) fn insert_internal(&self, flag: u32) {
		debug_assert_eq!(flag & Self::USER_FLAGS_MASK, 0, "attempted to set user flags.");

		self.0.fetch_or(flag & !Self::USER_FLAGS_MASK, Ordering::SeqCst);
	}

	/// Gets the list of flags.
	pub fn get(&self) -> u32 {
		self.0.load(Ordering::SeqCst)
	}

	/// Gets all user-defined flags
	pub fn get_user(&self) -> u32 {
		self.0.load(Ordering::SeqCst) & Self::USER_FLAGS_MASK
	}

	/// Masks the flags.
	pub fn mask(&self, mask: u32) -> u32 {
		self.get() & mask
	}

	/// Check to see if all the flags are set.
	pub fn contains(&self, flag: u32) -> bool {
		self.mask(flag) == flag
	}

	/// Checks to see if any of the flags are set.
	pub fn contains_any(&self, flag: u32) -> bool {
		self.mask(flag) != 0
	}

	/// Removes a user-defined flag.
	pub fn remove_user(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & !Self::USER_FLAGS_MASK, 0, "attempted to set non-user flags.");

		// TODO: is this the right way to remove
		self.0.fetch_and(!(flag & Self::USER_FLAGS_MASK), Ordering::SeqCst) & flag != 0
	}

	/// Removes an internal flag.
	pub(crate) fn remove_internal(&self, flag: u32) -> bool {
		debug_assert_eq!(flag & Self::USER_FLAGS_MASK, 0, "attempted to set user flags.");

		// FIXME: bitwise flag with user flags mask, but is it right?
		self.0.fetch_and(!(flag & !Self::USER_FLAGS_MASK), Ordering::SeqCst) & flag != 0
	}
}

impl Debug for Flags {
	#[allow(clippy::cognitive_complexity)]
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
			_UNUSED_21 _UNUSED_22 _UNUSED_23 _UNUSED_24 _UNUSED_25 _UNUSED_26 _UNUSED_27
			TYPE_FLAG1 TYPE_FLAG2 TYPE_FLAG3 TYPE_FLAG4
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
