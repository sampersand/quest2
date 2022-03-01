#[derive(Debug, Default, Clone)]
pub struct Basic { _priv: () }

impl Basic {
	pub const fn new() -> Self {
		Self { _priv: () }
	}
}

