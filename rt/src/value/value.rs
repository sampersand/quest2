use crate::value::base::{Attribute, HasDefaultParent};
use crate::value::ty::{AttrConversionDefined, List, Wrap, Integer, Float, RustFn, Text, Block};
use crate::value::{Convertible, Gc};
use crate::vm::Args;
use crate::{Result, Error};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU64;

/*
000...000 0000 = undefined
XXX...XXX 0000 = `Base<T>` (nonzero `X`)
XXX...XXX 1000 = rustfn (nonzero `X`, gotta remove the `1`)
XXX...XXX XXX1 = i63
XXX...XXX XX10 = f62
000...000 0100 = false
000...000 1000 = null
000...000 1100 = true

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
	#[must_use]
	pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
		Self::from_bits(NonZeroU64::new_unchecked(bits))
	}

	#[inline]
	#[must_use]
	pub const unsafe fn from_bits(bits: NonZeroU64) -> Self {
		Self(bits, PhantomData)
	}

	#[inline]
	#[must_use]
	pub const fn any(self) -> AnyValue {
		unsafe { std::mem::transmute(self) }
	}

	pub const fn id(self) -> u64 {
		self.0.get() // unique id for each object, technically lol
	}

	#[must_use]
	pub const fn is_allocated(self) -> bool {
		self.bits() & 0b1111 == 0
	}
}


impl<T: Convertible> Value<T> {
	#[must_use]
	pub fn get(self) -> T {
		T::get(self)
	}
}

pub struct Any {
	_priv: (),
}
pub type AnyValue = Value<Any>;

impl Default for AnyValue {
	fn default() -> Self {
		Value::NULL.any()
	}
}

impl AnyValue {
	fn parents_for(self) -> AnyValue {
		use crate::value::ty::*;

		if self.is_a::<Integer>() {
			Integer::parent()
		} else if self.is_a::<Float>() {
			Float::parent()
		} else if self.is_a::<Boolean>() {
			Boolean::parent()
		} else if self.is_a::<Null>() {
			Null::parent()
		} else if self.is_a::<RustFn>() {
			RustFn::parent()
		} else {
			unreachable!("called `parents_for` for an invalid type: {:064b}", self.bits())
		}
	}
}

impl AnyValue {
	fn allocate_self_and_copy_data_over(self) -> Self {
		use crate::value::ty::*;

		fn allocate_thing<T: 'static + HasDefaultParent>(thing: T) -> AnyValue {
			Value::from(Wrap::new(thing)).any()
		}

		if let Some(i) = self.downcast::<Integer>() {
			allocate_thing(i)
		} else if let Some(f) = self.downcast::<Float>() {
			allocate_thing(f)
		} else if let Some(b) = self.downcast::<Boolean>() {
			allocate_thing(b)
		} else if let Some(n) = self.downcast::<Null>() {
			allocate_thing(n)
		} else if let Some(f) = self.downcast::<RustFn>() {
			allocate_thing(f)
		} else {
			unreachable!("unrecognized copy type: {:064b}", self.bits())
		}
	}

	unsafe fn get_gc_any_unchecked(self) -> Gc<Wrap<Any>> {
		debug_assert!(self.is_allocated());

		Gc::new_unchecked(self.bits() as usize as *mut _)
	}

	pub fn has_attr<A: Attribute>(self, attr: A) -> Result<bool> {
		self.get_attr(attr).map(|opt| opt.is_some())
	}

	pub fn get_attr<A: Attribute>(self, attr: A) -> Result<Option<AnyValue>> {
		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }
				.as_ref()?
				.get_attr(attr)
		} else {
			self.parents_for().get_attr(attr)
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: AnyValue) -> Result<()> {
		if !self.is_allocated() {
			*self = self.allocate_self_and_copy_data_over();
		}

		unsafe { self.get_gc_any_unchecked() }
			.as_mut()?
			.set_attr(attr, value)
	}

	pub fn del_attr<A: Attribute>(self, attr: A) -> Result<Option<AnyValue>> {
		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }
				.as_mut()?
				.del_attr(attr)
		} else {
			Ok(None) // we don't delete from unallocated things.
		}
	}

	pub fn parents(&mut self) -> Result<Gc<List>> {
		if !self.is_allocated() {
			*self = self.allocate_self_and_copy_data_over();
		}

		Ok(unsafe { self.get_gc_any_unchecked() }.as_mut()?.parents_list())
	}

	pub fn call_attr<A: Attribute>(self, attr: A, args: Args<'_>) -> Result<AnyValue> {
		if self.is_allocated() {
			return unsafe { self.get_gc_any_unchecked() }.call_attr(attr, args);
		}

		// self.parents_for().call_attr(self, attr, args)
		todo!();
	}

	pub fn call(self, obj: AnyValue, args: Args<'_>) -> Result<AnyValue> {
		if let Some(rustfn) = self.downcast::<RustFn>() {
			rustfn.call(obj, args)
		} else if let Some(block) = self.downcast::<Gc<Block>>() {
			block.as_ref()?.call(obj, args)
		} else {
			// gotta add `self` to the front
			self.call_attr("()", args)
		}
	}
}

impl AnyValue {
	pub fn to_boolean(self) -> Result<bool> {
		Ok(self.bits() == Value::NULL.bits() || self.bits() == Value::FALSE.bits())
	}

	pub fn to_integer(self) -> Result<Integer> {
		let bits = self.bits();

		if self.is_allocated() {
			return self.convert::<Integer>();
		}

		if let Some(integer) = self.downcast::<Integer>() {
			Ok(integer)
		} else if let Some(float) = self.downcast::<Float>() {
			Ok(float as Integer)
		} else if bits <= Value::NULL.bits() {
			debug_assert!(bits == Value::NULL.bits() || bits == Value::FALSE.bits());
			Ok(0)
		} else if bits == Value::TRUE.bits() {
			Ok(1)
		} else {
			debug_assert!(self.is_a::<RustFn>());
			Err(Error::ConversionFailed(self, Integer::ATTR_NAME))
		}
	}

	pub fn to_text(self) -> Result<Gc<Text>> {
		if let Some(text) = self.downcast::<Gc<Text>>() {
			return Ok(text);
		}

		todo!()
		// let bits = self.bits();

		// if self.is_allocated() {
		// 	return self.convert::<Integer>();
		// }

		// if let Some(integer) = self.downcast::<Integer>() {
		// 	Ok(integer)
		// } else if let Some(float) = self.downcast::<Float>() {
		// 	Ok(float.get() as Integer)
		// } else if bits <= Value::NULL.bits() {
		// 	debug_assert!(bits == Value::NULL.bits() || bits == Value::FALSE.bits());
		// 	Ok(0)
		// } else if bits == Value::TRUE.bits() {
		// 	Ok(1)
		// } else {
		// 	debug_assert!(self.is_a::<RustFn>());
		// 	Err(Error::ConversionFailed(self, Integer::ATTR_NAME))
		// }
	}

	pub fn convert<C: AttrConversionDefined + Convertible>(self) -> Result<C> {
		let conv = self.call_attr(C::ATTR_NAME, Default::default())?;

		if let Some(attr) = conv.downcast::<C>() {
			Ok(attr)
		} else {
			Err(Error::ConversionFailed(conv, C::ATTR_NAME))
		}
	}
}

impl AnyValue {
	#[must_use]
	pub fn is_a<T: Convertible>(self) -> bool {
		T::is_a(self)
	}

	#[must_use]
	pub fn downcast<T: Convertible>(self) -> Option<T> {
		T::downcast(self).map(T::get)
	}

	pub fn try_hash(self) -> Result<u64> {
		if self.is_allocated() {
			if let Some(text) = self.downcast::<Gc<Text>>() {
				// OPTIMIZE ME!
				use std::hash::{Hash, Hasher};
				use std::collections::hash_map::DefaultHasher;

				let mut s = DefaultHasher::new();
				text.as_ref()?.hash(&mut s);
				return Ok(s.finish())
			}
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

impl<T: Convertible + Debug> Debug for Value<T> {
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

impl Debug for crate::value::gc::Ref<Wrap<Any>> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&Value::from(self.as_gc()), f)
	}
}
