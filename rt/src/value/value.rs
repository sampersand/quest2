use crate::Result;
use crate::value::{Convertible, Gc, ty::Wrap};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU64;
use crate::value::base::HasParents;

/*
000...0 000 000 = undefined
XXX...X XXX 000 = pointer (nonzero `X`)
XXX...X XXX XX1 = i63
XXX...X XXX X10 = f62
XXX...X XXX 100 = rustfn (nonzero `X`, gotta remove the `1`)
000...0 001 100 = false
000...0 010 100 = true
000...0 011 100 = null

NOTE: Technically, the first page can be allocated in some architectures
(and thus `false`/`true`/`null` constants could ~technically~ be allocated).
However, those are microkernels so I dont really care. No relevant OS will
map the first page to userspace.
*/
#[repr(transparent)]
pub struct Value<T>(NonZeroU64, PhantomData<T>);

sa::assert_eq_size!(Value<i64>, Value<[u64; 64]>, AnyValue);
sa::assert_eq_align!(Value<i64>, Value<[u64; 64]>, AnyValue);

sa::assert_eq_size!(AnyValue, u64, *const (), Option<AnyValue>);
sa::assert_eq_align!(AnyValue, u64, *const (), Option<AnyValue>);

sa::assert_not_impl_any!(AnyValue: Drop);
sa::assert_not_impl_any!(Value<i64>: Drop);

impl<T> Copy for Value<T> {}
impl<T> Clone for Value<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Value<T> {
	#[inline]
	pub const fn bits(self) -> u64 {
		self.0.get()
	}

	#[inline]
	pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
		Self::from_bits(NonZeroU64::new_unchecked(bits))
	}

	#[inline]
	pub const unsafe fn from_bits(bits: NonZeroU64) -> Self {
		Self(bits, PhantomData)
	}

	#[inline]
	pub const fn any(self) -> AnyValue {
		unsafe { std::mem::transmute(self) }
	}

	pub const fn id(self) -> u64 {
		self.0.get() // unique id for each object, technically lol
	}

	pub const fn is_allocated(self) -> bool {
		self.bits() & 0b111 == 0
	}
}


impl AnyValue {
	fn parents_for(self) -> super::base::Parents {
		use crate::value::ty::*;

		match self.bits() {
			b if b & 1 == 1 => Integer::parents(),
			b if b & 2 == 2 => Float::parents(),
			b if b == Value::TRUE.bits() || b == Value::FALSE.bits() => Boolean::parents(),
			b if b == Value::NULL.bits() => Null::parents(),
			b if b & 8 == 8 => RustFn::parents(),
			b if b & 7 == 0 => unreachable!("called parents_for on allocated"),
			b => unreachable!("unknown bits? {:064b}", b)
		}
	}
}

impl AnyValue {
	fn allocate_self_and_copy_data_over(self) -> AnyValue {
		use crate::value::ty::*;

		fn allocate_thing<T: 'static + HasParents>(thing: T) -> AnyValue {
			Value::from(Wrap::new(thing)).any()
		}

		if let Some(i) = self.downcast::<Integer>() {
			allocate_thing(i.get())
		} else if let Some(f) = self.downcast::<Float>() {
			allocate_thing(f.get())
		} else if let Some(b) = self.downcast::<Boolean>() {
			allocate_thing(b.get())
		} else if let Some(n) = self.downcast::<Null>() {
			allocate_thing(n.get())
		} else if let Some(f) = self.downcast::<RustFn>() {
			allocate_thing(f.get())
		} else {
			unreachable!("unrecognized copy type: {:064b}", self.bits())
		}
	}

	unsafe fn get_gc_any_unchecked(self) -> Gc<Wrap<Any>> {
		debug_assert!(self.is_allocated());
		
		Gc::new_unchecked(self.bits() as usize as *mut _)
	}

	pub fn has_attr(self, attr: AnyValue) -> Result<bool> {
		self.get_attr(attr).map(|opt| opt.is_some())
	}

	pub fn get_attr(self, attr: AnyValue) -> Result<Option<AnyValue>> {
		if self.is_allocated() {
			return unsafe { self.get_gc_any_unchecked() }.as_ref()?.get_attr(attr);
		} else {
			self.parents_for().get_attr(attr)
		}
	}
	
	pub fn set_attr(&mut self, attr: AnyValue, value: AnyValue) -> Result<()> {
		if !self.is_allocated() {
			*self = self.allocate_self_and_copy_data_over();
		}

		unsafe { self.get_gc_any_unchecked() }.as_mut()?.set_attr(attr, value)
	}

	pub fn del_attr(self, attr: AnyValue) -> Result<Option<AnyValue>> {
		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }.as_mut()?.del_attr(attr)
		} else {
			Ok(None) // we don't delete from unallocated things.
		}
	}

	pub fn parents(&mut self) -> Result<Gc<crate::value::ty::List>> {
		if !self.is_allocated() {
			*self = self.allocate_self_and_copy_data_over();
		}

		Ok(unsafe { self.get_gc_any_unchecked() }.as_mut()?.parents())
	}
}

impl<T: Convertible> Value<T> {
	pub fn get(self) -> T::Output {
		T::get(self)
	}
}

pub struct Any { _priv: () }
pub type AnyValue = Value<Any>;

impl AnyValue {
	pub fn is_a<T: Convertible>(self) -> bool {
		T::is_a(self)
	}

	pub fn downcast<T: Convertible>(self) -> Option<Value<T>> {
		T::downcast(self)
	}

	pub fn try_hash(self) -> Result<u64> {
		if self.is_allocated() {
			todo!()
		} else {
			Ok(self.bits()) // this can also be modified, but that's a future thing
		}
	}

	pub fn try_eq(self, rhs: AnyValue) -> Result<bool> {
		if self.is_allocated() {
			todo!()
		} else {
			Ok(self.bits() == rhs.bits()) // this can also be modified, but that's a future thing
		}
	}
}

impl<T: Convertible> Debug for Value<T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("Value(")?;

		Debug::fmt(&T::get(*self), f)?;

		f.write_str(")")
	}
}

impl Debug for AnyValue {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		use crate::value::ty::*;

		if let Some(i) = self.downcast::<Integer>() {
			Debug::fmt(&i, fmt)
		} else if let Some(f) = self.downcast::<Float>() {
			Debug::fmt(&f, fmt)
		} else if let Some(b) = self.downcast::<Boolean>() {
			Debug::fmt(&b, fmt)
		} else if let Some(n) = self.downcast::<Null>() {
			Debug::fmt(&n, fmt)
		} else if let Some(f) = self.downcast::<RustFn>() {
			Debug::fmt(&f, fmt)
		} else if let Some(t) = self.downcast::<Gc<Text>>() {
			Debug::fmt(&t, fmt)
		} else if let Some(l) = self.downcast::<Gc<List>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(i) = self.downcast::<Gc<Wrap<Integer>>>() {
			Debug::fmt(&i, fmt)
		} else if let Some(f) = self.downcast::<Gc<Wrap<Float>>>() {
			Debug::fmt(&f, fmt)
		} else if let Some(b) = self.downcast::<Gc<Wrap<Boolean>>>() {
			Debug::fmt(&b, fmt)
		} else if let Some(n) = self.downcast::<Gc<Wrap<Null>>>() {
			Debug::fmt(&n, fmt)
		} else if let Some(f) = self.downcast::<Gc<Wrap<RustFn>>>() {
			Debug::fmt(&f, fmt)
		} else {
			write!(fmt, "Value(<unknown:{:p}>)", self.0.get() as *const ())
		}
	}
}

impl Debug for crate::value::gc::GcRef<Wrap<Any>> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&Value::from(self.as_gc()), f)
	}
}
