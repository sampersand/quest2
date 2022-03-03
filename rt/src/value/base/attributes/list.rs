use crate::value::AnyValue;
use crate::Result;

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct ListMap(Box<Vec<(AnyValue, AnyValue)>>);

impl ListMap {
	pub fn into_inner(self) -> Box<Vec<(AnyValue, AnyValue)>> {
		self.0
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn get_attr(&self, attr: AnyValue) -> Result<Option<AnyValue>> {
		for &(key, val) in self.0.iter() {
			if attr.try_eq(key)? {
				return Ok(Some(val))
			}
		}

		Ok(None)
	}

	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		for (key, val) in self.0.iter_mut() {
			if attr.try_eq(*key)? {
				*val = value;
				return Ok(());
			}
		}

		self.0.push((attr, value));
		Ok(())
	}

	pub fn del_attr(&mut self, attr: AnyValue) -> Result<Option<AnyValue>> {
		let mut idx = None;
		for (i, (key, _)) in self.0.iter().enumerate() {
			if attr.try_eq(*key)? {
				idx = Some(i);
				break;
			}
		}

		if let Some(idx) = idx {
			Ok(Some(self.0.swap_remove(idx).1))
		} else {
			Ok(None)
		}
	}
}
