use crate::value::base::{Attribute, HasDefaultParent};
use crate::value::ty::{AttrConversionDefined, BoundFn, Float, Integer, List, RustFn, Text, Wrap};
use crate::value::{Convertible, Gc, Intern, ToAny};
use crate::vm::{Args, Block};
use crate::Result;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU64;

/*
000...0000 0000 = undefined
XXX...XXXX 0000 = `Base<T>` (nonzero `X`)
XXX...XXXX 1000 = RustFn (nonzero `X`, gotta remove the `1`)
XXX...XXXX XXX1 = i63
XXX...XXXX XX10 = f62
000...0000 0100 = false
000...0001 0100 = null
000...0010 0100 = true
XXX...X1Y0 0100 = undefined, `Y` is used to indicate frozen in InternKey.

Note that the `XXX...X100 0100` variant is used to make it so a union between `AnyValue` and
`Intern` will be unambiguously one type or the other.

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
	#[must_use]
	pub const fn bits(self) -> u64 {
		self.0.get()
	}

	#[inline]
	#[must_use]
	pub const fn raw_bits(self) -> NonZeroU64 {
		self.0
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

	#[must_use]
	pub const fn id(self) -> u64 {
		self.0.get() // unique id for each object, technically lol
	}

	#[must_use]
	#[allow(clippy::verbose_bit_mask)] // makes more sense this way...
	pub const fn is_allocated(self) -> bool {
		self.bits() & 0b1111 == 0
	}

	#[inline]
	#[must_use]
	pub const fn is_identical<U>(self, rhs: Value<U>) -> bool {
		self.bits() == rhs.bits()
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
	fn parents_for(self) -> Self {
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

	#[must_use]
	pub fn typename(self) -> &'static str {
		use crate::value::ty::*;

		match () {
			_ if self.is_a::<Integer>() || self.is_a::<Gc<Wrap<Integer>>>() => "Integer",
			_ if self.is_a::<Float>() || self.is_a::<Gc<Wrap<Float>>>() => "Float",
			_ if self.is_a::<Boolean>() || self.is_a::<Gc<Wrap<Boolean>>>() => "Boolean",
			_ if self.is_a::<Null>() || self.is_a::<Gc<Wrap<Null>>>() => "Null",
			_ if self.is_a::<RustFn>() || self.is_a::<Gc<Wrap<RustFn>>>() => "RustFn",
			_ if self.is_a::<Gc<Text>>() => "Text",
			_ if self.is_a::<Gc<List>>() => "List",
			_ if self.is_a::<Gc<Class>>() => "Class",
			_ if self.is_a::<Gc<Scope>>() => "Scope",
			_ if self.is_a::<Gc<BoundFn>>() => "BoundFn",
			_ if self.is_a::<Gc<crate::vm::Block>>() => "Block",
			_ if self.is_a::<Gc<crate::vm::Frame>>() => "Frame",
			_ if cfg!(debug_assertions) => panic!("todo: typename for {:?}", self),
			_ => "(Unknown)",
		}
	}

	pub fn dbg_text(self) -> Result<Gc<Text>> {
		self
			.call_attr(Intern::dbg, Args::default())?
			.try_downcast::<Gc<Text>>()
	}

	pub fn is_truthy(self) -> Result<bool> {
		self.to_boolean()
	}

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

	pub fn freeze(self) -> Result<()> {
		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }.as_ref()?.freeze();
		}

		Ok(())
	}

	pub fn has_attr<A: Attribute>(self, attr: A) -> Result<bool> {
		self.get_unbound_attr(attr).map(|opt| opt.is_some())
	}

	pub fn try_get_attr<A: Attribute>(self, attr: A) -> Result<Self> {
		self.get_attr(attr)?
			.ok_or_else(|| crate::error::ErrorKind::UnknownAttribute(self, attr.to_value()).into())
	}

	pub fn get_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
		let value = if let Some(value) = self.get_unbound_attr(attr)? {
			value
		} else {
			return Ok(None);
		};

		// If the value is callable, wrap it in a bound fn.
		if value.is_a::<RustFn>() || value.has_attr(Intern::op_call)? {
			Ok(Some(BoundFn::new(self, value).to_any()))
		} else {
			Ok(Some(value))
		}
	}

	pub fn try_get_unbound_attr<A: Attribute>(self, attr: A) -> Result<Self> {
		self.get_unbound_attr(attr)?
			.ok_or_else(|| crate::error::ErrorKind::UnknownAttribute(self, attr.to_value()).into())
	}

	pub fn get_unbound_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
		if !self.is_allocated() {
			return if attr.is_parents() {
				// TODO: if this is modified, it wont reflect on the integer.
				// so make `get_unbound_attr` require a reference?
				Ok(Some(List::from_slice(&[self.parents_for()]).to_any()))
			} else {
				self.parents_for().get_unbound_attr(attr)
			};
		}

		let gc = unsafe { self.get_gc_any_unchecked() };

		// 99% of the time it's not special.
		if !attr.is_special() {
			return gc.as_ref()?.get_unbound_attr(attr);
		}

		if attr.is_parents() {
			Ok(Some(gc.as_mut()?.parents_list().to_any()))
		} else {
			unreachable!("unknown special attribute");
		}
	}

	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Self) -> Result<()> {
		if !self.is_allocated() {
			*self = self.allocate_self_and_copy_data_over();
		}

		let gc = unsafe { self.get_gc_any_unchecked() };

		// 99% of the time it's not special.
		if !attr.is_special() {
			return gc.as_mut()?.set_attr(attr, value);
		}

		if attr.is_parents() {
			if let Some(list) = value.downcast::<Gc<List>>() {
				gc.as_mut()?.set_parents(list);

				Ok(())
			} else {
				Err(crate::error::ErrorKind::Message("can only set __parents__ to a List".to_string()).into())
			}
		} else {
			unreachable!("unknown special attribute");
		}
	}

	pub fn del_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
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

		Ok(unsafe { self.get_gc_any_unchecked() }
			.as_mut()?
			.parents_list())
	}

	pub fn call_attr<A: Attribute>(self, attr: A, args: Args<'_>) -> Result<Self> {
		if self.is_allocated() {
			return unsafe { self.get_gc_any_unchecked() }.call_attr(attr, args);
		}

		// OPTIMIZE ME: This is circumventing potential optimizations from `parents_for`?
		self
			.parents_for()
			.get_unbound_attr(attr)?
			.ok_or_else(|| crate::error::ErrorKind::UnknownAttribute(self, attr.to_value()))?
			.call(args.with_self(self))
	}

	// there's a potential logic flaw here, as this may actually pass `self`
	// when calling `Intern::op_call`. todo, check that out.
	pub fn call(self, args: Args<'_>) -> Result<Self> {
		if let Some(rustfn) = self.downcast::<RustFn>() {
			return rustfn.call(args);
		}

		if let Some(block) = self.downcast::<Gc<Block>>() {
			return block.run(args);
		}

		if let Some(boundfn) = self.downcast::<Gc<BoundFn>>() {
			return boundfn.qs_call(args);
		}

		self.call_attr(Intern::op_call, args)
	}

	pub fn to_boolean(self) -> Result<bool> {
		Ok(self.bits() != Value::NULL.bits() && self.bits() != Value::FALSE.bits())
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
			Err(crate::error::ErrorKind::ConversionFailed(self, Integer::ATTR_NAME).into())
		}
	}

	// deprecated
	pub fn to_text(self) -> Result<Gc<Text>> {
		self.convert::<Gc<Text>>()
	}

	pub fn convert<C: AttrConversionDefined + Convertible>(self) -> Result<C> {
		if let Some(this) = self.downcast::<C>() {
			return Ok(this);
		}

		let conv = self.call_attr(C::ATTR_NAME, Args::default())?;

		if let Some(attr) = conv.downcast::<C>() {
			Ok(attr)
		} else {
			Err(crate::error::ErrorKind::ConversionFailed(conv, C::ATTR_NAME).into())
		}
	}

	#[must_use]
	pub fn is_a<T: Convertible>(self) -> bool {
		T::is_a(self)
	}

	#[must_use]
	pub fn downcast<T: Convertible>(self) -> Option<T> {
		T::downcast(self).map(T::get)
	}

	pub fn try_downcast<T: Convertible + crate::value::NamedType>(self) -> Result<T> {
		self
			.downcast()
			.ok_or_else(|| crate::error::ErrorKind::InvalidTypeGiven {
				expected: T::TYPENAME,
				given: self.typename(),
			}.into())
	}

	pub fn try_hash(self) -> Result<u64> {
		if self.is_allocated() {
			if let Some(text) = self.downcast::<Gc<Text>>() {
				Ok(text.as_ref()?.fast_hash())
			} else {
				// self.call_attr(Intern::hash, Args::default())?;
				todo!()
			}
		} else {
			Ok(self.bits()) // this can also be modified, but that's a future thing
		}
	}

	pub fn try_eq(self, rhs: Self) -> Result<bool> {
		if self.is_identical(rhs) {
			return Ok(true);
		}

		if self.is_allocated() {
			if let (Some(lhs), Some(rhs)) = (self.downcast::<Gc<Text>>(), rhs.downcast::<Gc<Text>>()) {
				Ok(*lhs.as_ref()? == *rhs.as_ref()?)
			} else {
				self
					.call_attr(Intern::op_eql, Args::new(&[rhs], &[]))?
					.convert()
			}
		} else {
			Ok(false)
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

		struct StructDebug<T>(T, &'static str);
		impl<T: Debug> Debug for StructDebug<T> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				if f.alternate() {
					write!(f, "{}({:?})", self.1, self.0)
				} else {
					Debug::fmt(&self.0, f)
				}
			}
		}

		if let Some(i) = self.downcast::<Integer>() {
			Debug::fmt(&StructDebug(i, "Integer"), fmt)
		} else if let Some(f) = self.downcast::<Float>() {
			Debug::fmt(&StructDebug(f, "Float"), fmt)
		} else if let Some(b) = self.downcast::<Boolean>() {
			Debug::fmt(&StructDebug(b, "Boolean"), fmt)
		} else if let Some(n) = self.downcast::<Null>() {
			Debug::fmt(&StructDebug(n, "Null"), fmt)
		} else if let Some(f) = self.downcast::<RustFn>() {
			Debug::fmt(&f, fmt)
		} else if let Some(t) = self.downcast::<Gc<Text>>() {
			Debug::fmt(&t, fmt)
		} else if let Some(l) = self.downcast::<Gc<List>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(l) = self.downcast::<Gc<Class>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(l) = self.downcast::<Gc<Scope>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(l) = self.downcast::<Gc<BoundFn>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(l) = self.downcast::<Gc<crate::vm::Frame>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(l) = self.downcast::<Gc<crate::vm::Block>>() {
			Debug::fmt(&l, fmt)
		} else if let Some(i) = self.downcast::<Gc<Wrap<Integer>>>() {
			Debug::fmt(&StructDebug(i, "Integer"), fmt)
		} else if let Some(f) = self.downcast::<Gc<Wrap<Float>>>() {
			Debug::fmt(&StructDebug(f, "Float"), fmt)
		} else if let Some(b) = self.downcast::<Gc<Wrap<Boolean>>>() {
			Debug::fmt(&StructDebug(b, "Boolean"), fmt)
		} else if let Some(n) = self.downcast::<Gc<Wrap<Null>>>() {
			Debug::fmt(&StructDebug(n, "Null"), fmt)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::value::ty::{Boolean, Integer};

	macro_rules! args {
		($($pos:expr),*) => (args!($($pos),* ; ));
		($($kwn:literal => $kwv:expr),*) => (args!(; $($kwn => $kwv),*));
		($($pos:expr),* ; $($kwn:literal => $kwv:expr),*) => {
			Args::new(&[$(value!($pos)),*], &[$(($kwn, value!($kwv))),*])
		}
	}

	macro_rules! value {
		($lit:literal) => {
			$lit.to_any()
		};
		($name:expr) => {
			$name
		};
	}

	#[test]
	fn test_get_attr() {
		let greeting = value!("Hello, world");

		greeting
			.get_attr(Intern::concat)
			.unwrap()
			.unwrap()
			.call_attr(Intern::op_call, args!["!"])
			.unwrap();

		assert_eq!(
			"Hello, world!",
			greeting
				.downcast::<Gc<Text>>()
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
	}

	#[test]
	fn test_call_attrs() {
		let greeting = value!("Hello, world");
		greeting.call_attr(Intern::concat, args!["!"]).unwrap();
		greeting.call_attr(Intern::concat, args![greeting]).unwrap();

		assert_eq!(
			"Hello, world!Hello, world!",
			greeting
				.downcast::<Gc<Text>>()
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);

		assert!(greeting
			.call_attr(Intern::op_eql, args![greeting])
			.unwrap()
			.downcast::<Boolean>()
			.unwrap());

		let five = value!(5);
		let twelve = value!(12);
		assert_eq!(
			17,
			five
				.call_attr(Intern::op_add, args![twelve])
				.unwrap()
				.downcast::<Integer>()
				.unwrap()
		);

		let ff = value!(255)
			.call_attr(Intern::at_text, args!["base" => 16])
			.unwrap();
		assert_eq!(
			"ff",
			ff.downcast::<Gc<Text>>()
				.unwrap()
				.as_ref()
				.unwrap()
				.as_str()
		);
	}
}
