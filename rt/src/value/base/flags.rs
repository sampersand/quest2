use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct Flags(AtomicU32);

impl Flags {
	pub const USER0: u32 = (1 << 0);
	pub const USER1: u32 = (1 << 1);
	pub const USER2: u32 = (1 << 2);
	pub const USER3: u32 = (1 << 3);
	pub const USER4: u32 = (1 << 4);
	pub const USER5: u32 = (1 << 5);
	pub const USER6: u32 = (1 << 6);
	pub const USER7: u32 = (1 << 7);
	pub const USER8: u32 = (1 << 8);
	pub const USER9: u32 = (1 << 9);
	pub const USER10: u32 = (1 << 10);
	pub const USER11: u32 = (1 << 11);
	pub const USER12: u32 = (1 << 12);
	pub const USER13: u32 = (1 << 13);
	pub const USER14: u32 = (1 << 14);
	pub const USER15: u32 = (1 << 15);

	pub const FROZEN: u32 = (1 << 16);
	pub const NOFREE: u32 = (1 << 17);
	pub const GCMARK: u32 = (1 << 18);
	pub const ATTR_MAP: u32 = (1 << 19);
	pub const SINGLE_PARENT: u32 = (1 << 20);
	pub const UNUSEDA: u32 = (1 << 21);
	pub const UNUSED9: u32 = (1 << 22);
	pub const UNUSED8: u32 = (1 << 23);
	pub const UNUSED7: u32 = (1 << 24);
	pub const UNUSED6: u32 = (1 << 25);
	pub const UNUSED5: u32 = (1 << 26);
	pub const UNUSED4: u32 = (1 << 27);
	pub const UNUSED3: u32 = (1 << 28);
	pub const UNUSED2: u32 = (1 << 29);
	pub const UNUSED1: u32 = (1 << 30);
	pub const UNUSED0: u32 = (1 << 31);

	pub const fn new(flags: u32) -> Self {
		Self(AtomicU32::new(flags))
	}

	pub fn insert(&self, flag: u32) {
		self.0.fetch_or(flag, Ordering::SeqCst);
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

	pub fn remove(&self, flag: u32) {
		self.0.fetch_and(!flag, Ordering::SeqCst);
	}
}

impl Debug for Flags {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Flags({:032b})", self.0.load(Ordering::SeqCst))
	}
}

impl Clone for Flags {
	fn clone(&self) -> Self {
		Self::new(self.get())
	}
}
