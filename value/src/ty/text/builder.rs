use super::{Text, MAX_EMBEDDED_LEN, FLAG_EMBEDDED};
use crate::base::Base;

pub struct Builder(*mut Text);

impl Builder {
	pub fn new(cap: usize) -> Self {
		Self(Text::with_capacity(cap))
	}

	pub fn finish(self) -> Gc<Text> {
		unsafe {
			Gc::new(self.0)
		}
	}

		unsafe {
				builder: Base::<Text>::allocate(),
				ptr: std::ptr::null_mut(),
				cap,
				len: 0
			};

			if cap <= MAX_EMBEDDED_LEN {
				this.builder.base().flags().insert(FLAG_EMBEDDED);
				this.cap = MAX_EMBEDDED_LEN;
				this.ptr = this.builder.data().as_mut().embed.data.as_mut_ptr();
			} else {
				// FIXME: this is shouldn't be using `assume_init` but i have no wifi
				let mut this = this.data().assume_init_mut();
				this.alloc.cap = cap;
				this.alloc.ptr = alloc::alloc(alloc_ptr_layout(cap))
			}
		}

	}
}
