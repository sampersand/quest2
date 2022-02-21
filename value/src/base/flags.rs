use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct BaseFlags(AtomicU32);

impl BaseFlags {
	pub const USER1: u32         = 0b00000000_00000001;
	pub const USER2: u32         = 0b00000000_00000010;
	pub const USER3: u32         = 0b00000000_00000100;
	pub const USER4: u32         = 0b00000000_00001000;

	pub const FROZEN: u32        = 0b00000000_00010000;
	pub const MUT_BORROWED: u32  = 0b00000000_00100000;
	pub const MANY_PARENTS: u32  = 0b00000000_01000000;

	pub fn insert(&self, flag: u32) {
		self.0.fetch_or(flag, Ordering::SeqCst);
	}

	pub fn contains(&self, flag: u32) -> bool {
		self.0.load(Ordering::SeqCst) & flag != 0
	}

	pub fn remove(&self, flag: u32) {
		self.0.fetch_and(!flag, Ordering::SeqCst);
	}
}
