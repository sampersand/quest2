bitflags::bitflags! {
	#[repr(transparent)]
	pub struct BaseFlags: AtomicU64 {
		const USER1     = 0b0000_0001;
		const USER2     = 0b0000_0010;
		const USER3     = 0b0000_0100;
		const USER4     = 0b0000_1000;

		const FROZEN    = 0b0001_0000;
	}
}

impl BaseFlags {
	fn as_atomic(&self) -> &AtomicU64 {
		unsafe {
			std::mem::transmute::<&BaseFlags, &AtomicU64>(self)
		}
	}

	pub fn is_frozen(&self) -> bool {
		self.as_atomic().loa
	}
}
