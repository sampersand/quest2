use super::{AttributesGuard, ParentsGuard, DataMutGuard, DataRefGuard};
use std::ops::{Deref, DerefMut};

// temporary hack until we get everything working with the new scheme
pub struct GlobalBaseGuard<'a, DG> {
	pub(super) attributes: AttributesGuard<'a>,
	pub(super) parents: ParentsGuard<'a>,
	pub(super) data: DG
}

impl<DG> GlobalBaseGuard<'_, DG> {
	pub fn attributes(&self) -> &AttributesGuard<'_> {
		&self.attributes
	}

	pub fn parents(&self) -> &ParentsGuard<'_> {
		&self.parents
	}
}

impl<'a, T> Deref for GlobalBaseGuard<'a, DataRefGuard<'a, T>> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<'a, T> Deref for GlobalBaseGuard<'a, DataMutGuard<'a, T>> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<'a, T> DerefMut for GlobalBaseGuard<'a, DataMutGuard<'a, T>> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}
