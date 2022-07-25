use crate::Value;
use std::marker::PhantomData;

pub const MAX_POSITIONAL_ARITY: usize = 7;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Positional<'a, const N: usize> {
	ptr: *const [Value; N],
	_lifetime: PhantomData<&'a ()>,
}

impl<'a, const N: usize> Positional<'a, N> {
	pub const fn new(args: &'a [Value; N]) -> Self {
		assert!(N < MAX_POSITIONAL_ARITY);

		Self { ptr: args.as_ptr().cast(), _lifetime: PhantomData }
	}

	pub fn try_new(args: &'a [Value]) -> Option<Self> {
		args.try_into().map(Self::new).ok()
	}
}

impl<'a, const N: usize> std::ops::Index<usize> for Positional<'a, N> {
	type Output = Value;

	#[inline]
	fn index(&self, idx: usize) -> &Self::Output {
		// unsafe { &*(self.ptr).cast::<Value>().add(idx) }
		unsafe { &(*self.ptr)[idx] }
	}
}
