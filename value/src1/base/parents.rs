use crate::{AnyValue, Gc, Result};

#[allow(unused)]
pub(crate) union Parents {
	singular: Option<AnyValue>,
	many: Gc<()>
}

assert_eq_size!(Parents, u64);
assert_eq_align!(Parents, u64);

impl Parents {
	#[allow(unused)]
	pub(crate) unsafe fn as_slice(&self, is_singular: bool) -> Result<&[AnyValue]> {
		todo!()
		// if is_singular {
		// 	Ok(self.singular
		// 			.as_ref()
		// 			.map(std::slice::from_ref)
		// 			.unwrap_or(&[]))
		// } else {
		// 	let g = self.many.as_ref()?;
		// 	Ok(self.many.as_ref()?.as_slice())
		// }
	}
}
