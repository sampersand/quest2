use crate::value::base::{Attribute, HasDefaultParent};
use crate::value::ty::{
	AttrConversionDefined, Boolean, BoundFn, Float, Integer, List, RustFn, Text, Wrap,
};
use crate::value::{
	Attributed, AttributedMut, Callable, Convertible, Gc, Intern, NamedType, ToValue,
};
use crate::vm::{Args, Block};
use crate::{ErrorKind, Result};
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

Note that the `XXX...X100 0100` variant is used to make it so a union between `Value` and
`Intern` will be unambiguously one type or the other.

NOTE: Technically, the first page can be allocated in some architectures
(and thus `false`/`true`/`null` constants could ~technically~ be allocated).
However, those are microkernels so I dont really care. No relevant OS will
map the first page to userspace.
*/
/// Any representable value within Quest uses this type.
///
/// The default `Value` itself represents any value at all, whereas specific values (`Value<Foo>`)
/// indicate that the contexts are exactly that type.
#[repr(transparent)]
pub struct Value<T = Any>(NonZeroU64, PhantomData<T>);

sa::assert_eq_size!(Value<i64>, Value<[u64; 64]>, Value);
sa::assert_eq_align!(Value<i64>, Value<[u64; 64]>, Value);

sa::assert_eq_size!(Value, u64, *const (), Option<Value>);
sa::assert_eq_align!(Value, u64, *const (), Option<Value>);

sa::assert_not_impl_any!(Value: Drop);
sa::assert_not_impl_any!(Value<i64>: Drop);

impl<T> Copy for Value<T> {}
impl<T> Clone for Value<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl Default for Value {
	fn default() -> Self {
		Value::NULL.to_value()
	}
}

impl<T> ToValue for Value<T> {
	fn to_value(self) -> Value {
		self.to_value_const()
	}
}

impl<T> Value<T> {
	/// Gets the underlying bits associated with this value.
	///
	/// If two [`Value`]s have the same bits, they're identical.
	///
	/// # Examples
	/// ```
	/// # use quest::ToValue;
	/// let bits = 12.to_value().bits();
	/// assert_eq!(bits, 12.to_value().bits());
	/// ```
	#[must_use]
	pub const fn bits(self) -> u64 {
		self.0.get()
	}

	/// Creates a new [`Value`] from the underlying bits.
	///
	/// # Safety
	/// Calling this function safely requires the following to hold:
	/// - `bits` are a valid representation of a [`Value`].
	/// - `bits` are nonzero (no valid [`Value`] representation is zero, anyways).
	/// - If `bits` correspond to a [`Gc`]-allocated object, the object must not have been
	///   garbage collected.
	///
	/// # Examples
	/// ```
	/// # use quest::{Value, ToValue};
	/// let twelve = 12.to_value();
	///
	/// // SAFETY: we know that `bits` corresponds to a valid
	/// // `Value`, as it came from one.
	/// let twelve2 = unsafe { Value::<i64>::from_bits(twelve.bits()) };
	///
	/// assert!(twelve.is_identical(twelve2));
	/// ```
	#[must_use]
	pub const unsafe fn from_bits(bits: u64) -> Self {
		debug_assert!(bits != 0); // `debug_assert_ne!(u64, u64)` isn't const-stable

		Self(NonZeroU64::new_unchecked(bits), PhantomData)
	}

	/// This is identical to [`Value::to_value`], except it's const-usable.
	#[must_use]
	pub const fn to_value_const(self) -> Value {
		unsafe { std::mem::transmute(self) }
	}

	/// Gets a unique id for this value.
	///
	/// Currently, this is identical to [`Value::bits`], but that isn't guaranteed.
	///
	/// # Examples
	/// ```
	/// # use quest::ToValue;
	/// let hello = "hello".to_value();
	/// let hello2 = String::from("hello").to_value();
	///
	/// assert_ne!(hello.id(), hello2.id());
	/// ```
	#[must_use]
	pub const fn id(self) -> u64 {
		self.bits() // unique id for each object, technically lol
	}

	/// Checks to see whether `self` corresponds to an allocated object.
	///
	/// # Examples
	/// ```
	/// # use quest::ToValue;
	/// assert!(!12.to_value().is_allocated());
	/// assert!(!true.to_value().is_allocated());
	/// assert!("hello".to_value().is_allocated());
	/// ```
	#[must_use]
	#[allow(clippy::verbose_bit_mask)] // makes more sense this way...
	pub const fn is_allocated(self) -> bool {
		self.bits() & 0b1111 == 0
	}

	/// Checks to see whether `self` is identical to `rhs`, i.e. they are the same object.
	///
	/// # Examples
	/// ```
	/// # use quest::ToValue;
	/// let hello = "hello".to_value();
	/// let hello2 = String::from("hello").to_value();
	///
	/// assert!(hello.is_identical(hello));
	/// assert!(!hello.is_identical(hello2));
	/// ```
	#[must_use]
	pub const fn is_identical<U>(self, rhs: Value<U>) -> bool {
		self.id() == rhs.id()
	}
}

/// A struct that represents a [`Value`] of any type.
///
/// It will never be constructed directly, instead you must try to [`Value::downcast`]/
/// [`Value::try_downcast`] out of it.
///
/// This is created via [`ToValue::to_value`].
pub struct Any {
	_priv: (),
}

impl Value {
	/// Gets a debug representation of `self`.
	pub fn dbg_text(self) -> Result<Gc<Text>> {
		self.call_attr(Intern::dbg, Args::default())?.try_downcast::<Gc<Text>>()
	}

	/// Checks to see if `self` is truthy.
	///
	/// In Quest, only false and null are falsey.
	pub fn is_truthy(self) -> bool {
		self.bits() != Value::NULL.bits() && self.bits() != Value::FALSE.bits()
	}

	/// Gets the name of this type.
	#[must_use]
	pub fn typename(self) -> crate::value::Typename {
		use crate::value::ty::*;

		match () {
			_ if self.is_a::<Integer>() || self.is_a::<Gc<Wrap<Integer>>>() => Integer::TYPENAME,
			_ if self.is_a::<Float>() || self.is_a::<Gc<Wrap<Float>>>() => Float::TYPENAME,
			_ if self.is_a::<Boolean>() || self.is_a::<Gc<Wrap<Boolean>>>() => Boolean::TYPENAME,
			_ if self.is_a::<Null>() || self.is_a::<Gc<Wrap<Null>>>() => Null::TYPENAME,
			_ if self.is_a::<RustFn>() || self.is_a::<Gc<Wrap<RustFn>>>() => RustFn::TYPENAME,
			_ if self.is_a::<Gc<Text>>() => Gc::<Text>::TYPENAME,
			_ if self.is_a::<Gc<List>>() => Gc::<List>::TYPENAME,
			_ if self.is_a::<Gc<Class>>() => Gc::<Class>::TYPENAME,
			_ if self.is_a::<Gc<Scope>>() => Gc::<Scope>::TYPENAME,
			_ if self.is_a::<Gc<BoundFn>>() => Gc::<BoundFn>::TYPENAME,
			_ if self.is_a::<Gc<BigNum>>() => Gc::<BigNum>::TYPENAME,
			_ if self.is_a::<Gc<crate::vm::Block>>() => Gc::<crate::vm::Block>::TYPENAME,
			_ if self.is_a::<Gc<crate::vm::Frame>>() => Gc::<crate::vm::Frame>::TYPENAME,
			_ if cfg!(debug_assertions) => panic!("todo: typename for {:?}", self),
			_ => "(Unknown)",
		}
	}

	// SAFETY: must be called with an unallocated type.
	unsafe fn parents_for_unallocated(self) -> Self {
		use crate::value::ty::*;

		debug_assert!(!self.is_allocated());
		match () {
			_ if self.is_a::<Integer>() => Integer::parent(),
			_ if self.is_a::<Boolean>() => Boolean::parent(),
			_ if self.is_a::<Float>() => Float::parent(),
			_ if self.is_a::<Null>() => Null::parent(),
			_ if self.is_a::<RustFn>() => RustFn::parent(),
			_ if cfg!(debug_assertions) => unreachable!(
				"called `parents_for_unallocated` for an invalid type: {:064b}",
				self.bits()
			),
			_ => std::hint::unreachable_unchecked(),
		}
	}

	// SAFETY: must be called with an unallocated type.
	unsafe fn allocate_self_and_copy_data_over(self) -> Self {
		use crate::value::ty::*;

		debug_assert!(!self.is_allocated());

		if let Some(i) = self.downcast::<Integer>() {
			Wrap::new(i).to_value()
		} else if let Some(f) = self.downcast::<Float>() {
			Wrap::new(f).to_value()
		} else if let Some(b) = self.downcast::<Boolean>() {
			Wrap::new(b).to_value()
		} else if let Some(n) = self.downcast::<Null>() {
			Wrap::new(n).to_value()
		} else if let Some(f) = self.downcast::<RustFn>() {
			Wrap::new(f).to_value()
		} else if cfg!(debug_assertions) {
			unreachable!("unrecognized copy type: {:064b}", self.bits())
		} else {
			std::hint::unreachable_unchecked()
		}
	}

	unsafe fn get_gc_any_unchecked(self) -> Gc<Wrap<Any>> {
		debug_assert!(self.is_allocated());

		Gc::new_unchecked(self.bits() as usize as *mut _)
	}

	/// Freezes `self`, disallowing further mutable access.
	///
	/// # Errors
	/// Will return an error if `self` is currently mutably borrowed.
	pub fn freeze(self) -> Result<()> {
		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }.as_ref()?.freeze();
		}

		Ok(())
	}

	/// Gets the list of parents associated with `self`
	///
	/// This takes a mutable reference in case `self` is not allocated
	pub fn parents_list(&mut self) -> Result<Gc<List>> {
		if !self.is_allocated() {
			// SAFETY: `self` is unallocated, as we just verified
			unsafe {
				*self = self.allocate_self_and_copy_data_over();
			}
		}

		Ok(unsafe { self.get_gc_any_unchecked() }.as_mut()?.parents_list())
	}
}

impl crate::value::Attributed for Value {
	fn get_unbound_attr_checked<A: Attribute>(
		&self,
		attr: A,
		checked: &mut Vec<Self>,
	) -> Result<Option<Self>> {
		(*self).get_unbound_attr_checked(attr, checked)
	}

	fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		(*self).get_unbound_attr(attr)
	}

	fn get_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		(*self).get_attr(attr)
	}

	fn has_attr<A: Attribute>(&self, attr: A) -> Result<bool> {
		// (*self).has_attr(attr)
		self.get_unbound_attr(attr).map(|opt| opt.is_some())
	}

	fn try_get_attr<A: Attribute>(&self, attr: A) -> Result<Value> {
		(*self).try_get_attr(attr)
	}
}

impl AttributedMut for Value {
	fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value> {
		let _ = attr;
		todo!();
	}

	fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()> {
		self.set_attr(attr, value)
	}

	fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>> {
		(*self).del_attr(attr)
	}
}

impl Value {
	/// Checks to see if `self` has the attribute `attr`.
	pub fn has_attr<A: Attribute>(self, attr: A) -> Result<bool> {
		self.get_unbound_attr(attr).map(|opt| opt.is_some())
	}

	/// Attempts to get the attribute `attr`, returning `Err` if it doesn't exist.
	pub fn try_get_attr<A: Attribute>(self, attr: A) -> Result<Self> {
		self.get_attr(attr)?.ok_or_else(|| {
			ErrorKind::UnknownAttribute { object: self, attribute: attr.to_value() }.into()
		})
	}

	/// Get the attribute `attr`, returning `None` if it doesnt exist.
	///
	/// For attributes which have [`Intern::op_call`] defined on them, this will create a new
	/// [`BoundFn`]. For all other types, it just returns the attribute itself.
	pub fn get_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
		let value = if let Some(value) = self.get_unbound_attr(attr)? {
			value
		} else {
			return Ok(None);
		};

		let is_callable = value.is_a::<RustFn>()
			|| value.is_a::<Gc<Block>>()
			|| value.is_a::<Gc<BoundFn>>()
			|| value.has_attr(Intern::op_call)?;

		// If the value is callable, wrap it in a bound fn. Short circuit for common ones.
		if is_callable {
			return Ok(Some(BoundFn::new(self, value).to_value()));
		}

		Ok(Some(value))
	}

	/// Attempts to get the unbound attribute `attr`, returning `Err` if it doesn't exist.
	pub fn try_get_unbound_attr<A: Attribute>(self, attr: A) -> Result<Self> {
		self.get_unbound_attr(attr)?.ok_or_else(|| {
			ErrorKind::UnknownAttribute { object: self, attribute: attr.to_value() }.into()
		})
	}

	/// Get the unbound attribute `attr`, returning `None` if it doesnt exist.
	pub fn get_unbound_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
		self.get_unbound_attr_checked(attr, &mut Vec::new())
	}

	fn assert_isnt_an_objectified_frame(self) {
		#[cfg(debug_assertions)]
		if let Some(frame) = self.downcast::<Gc<crate::vm::Frame>>() {
			debug_assert!(
				frame.as_ref().map_or(true, |x| x.is_object()),
				"attempted to get an attr on an un-objectified frame."
			);
		}
	}

	/// Get the unbound attribute `attr`, with a list of checked parents, `None` if it doesnt exist.
	///
	/// The `checked` parameter allows us to keep track of which parents have already been checked,
	/// so as to prevent checking the same parents more than once.
	pub fn get_unbound_attr_checked<A: Attribute>(
		self,
		attr: A,
		checked: &mut Vec<Self>,
	) -> Result<Option<Self>> {
		self.assert_isnt_an_objectified_frame();

		if !self.is_allocated() {
			let parents = unsafe { self.parents_for_unallocated() };

			// 99% of the time it's not special.
			if !attr.is_special() {
				return parents.get_unbound_attr_checked(attr, checked);
			}

			if attr.is_parents() {
				// TODO: if this is modified, it wont reflect on the integer.
				// so make `get_unbound_attr` require a reference?
				return Ok(Some(List::from_slice(&[parents]).to_value()));
			} else {
				unreachable!("unknown special attribute: {attr:?}");
			}
		}

		let gc = unsafe { self.get_gc_any_unchecked() };

		// 99% of the time it's not special.
		if !attr.is_special() {
			return gc.as_ref()?.get_unbound_attr_checked(attr, checked);
		}

		if attr.is_parents() {
			Ok(Some(gc.as_mut()?.parents_list().to_value()))
		} else {
			unreachable!("unknown special attribute: {attr:?}");
		}
	}

	/// Sets the attribute `attr` on `self` to `value`.
	///
	/// Note that this takes a mutable reference to `self` in case `self` is not an allocated type:
	/// If it isn't, `self` will be replaced with an allocated version, with the attribute set on
	/// that type.
	pub fn set_attr<A: Attribute>(&mut self, attr: A, value: Self) -> Result<()> {
		self.assert_isnt_an_objectified_frame();

		if !self.is_allocated() {
			// SAFETY: `self` is unallocated, as we just verified
			unsafe {
				*self = self.allocate_self_and_copy_data_over();
			}
		}

		unsafe { self.get_gc_any_unchecked() }.as_mut()?.set_attr(attr, value)
	}

	/// Deletes the attribute `attr` from `self`, returning whatever was there before.
	///
	/// Note that unallocated types don't actually have attributes defined on them, so they always
	/// will return `Ok(None)`
	pub fn del_attr<A: Attribute>(self, attr: A) -> Result<Option<Self>> {
		self.assert_isnt_an_objectified_frame();

		if self.is_allocated() {
			unsafe { self.get_gc_any_unchecked() }.as_mut()?.del_attr(attr)
		} else {
			Ok(None) // we don't delete from unallocated things.
		}
	}

	/// Calls the attribute `attr` on `self` with the given arguments. If the attr doesnt exist,
	/// it raises an error
	pub fn call_attr<A: Attribute>(self, attr: A, args: Args<'_>) -> Result<Self> {
		if self.is_allocated() {
			return unsafe { self.get_gc_any_unchecked() }.call_attr(attr, args);
		}

		// OPTIMIZE ME: This is circumventing potential optimizations from `parents_for_unallocated`?
		unsafe { self.parents_for_unallocated() }
			.try_get_unbound_attr(attr)?
			.call(args.with_this(self))
	}
}

impl Callable for Value {
	/// Calls `self` with the given `args`.
	///
	/// Equivalent to `self.call_attr(Intern::op_call, args)`, except with optimizations for common
	/// types.
	fn call(self, args: Args<'_>) -> Result<Self> {
		// there's a potential logic flaw here, as this may actually pass `self`
		// when calling `Intern::op_call`. todo, check that out.
		if let Some(rustfn) = self.downcast::<RustFn>() {
			return rustfn.call(args);
		}

		if let Some(block) = self.downcast::<Gc<Block>>() {
			return block.run(args);
		}

		if let Some(boundfn) = self.downcast::<Gc<BoundFn>>() {
			return boundfn.call(args);
		}

		self.call_attr(Intern::op_call, args)
	}
}

impl Value {
	/// Converts `self` to a [`Boolean`], but with optimizations for builtin types.
	pub fn to_boolean(self) -> Result<Boolean> {
		Ok(self.bits() != Value::NULL.bits() && self.bits() != Value::FALSE.bits())
	}

	/// Converts `self` to an [`Integer`], but with some optimizations for builtin types.
	pub fn to_integer(self) -> Result<Integer> {
		let bits = self.bits();

		if self.is_allocated() {
			return self.convert::<Integer>();
		}

		if let Some(integer) = self.downcast::<Integer>() {
			Ok(integer)
		} else if let Some(float) = self.downcast::<Float>() {
			Ok(Integer::new_truncate(float as i64))
		} else if bits <= Value::NULL.bits() {
			debug_assert!(bits == Value::NULL.bits() || bits == Value::FALSE.bits());
			Ok(Integer::ZERO)
		} else if bits == Value::TRUE.bits() {
			Ok(Integer::ONE)
		} else {
			debug_assert!(self.is_a::<RustFn>());
			Err(ErrorKind::ConversionFailed { object: self, into: Integer::TYPENAME }.into())
		}
	}

	/// Converts `self` to a [`Text`], but with some optimizations for builtin types.
	pub fn to_text(self) -> Result<Gc<Text>> {
		// TODO: optimize
		self.convert::<Gc<Text>>()
	}

	/// Converts `self` to a [`List`], but with some optimizations for builtin types.
	pub fn to_list(self) -> Result<Gc<List>> {
		// TODO: optimize
		self.convert::<Gc<List>>()
	}

	/// Converts `self` to a given type with the conversion defined.
	///
	/// Note that if you're converting to an [`Integer`], [`Text`], [`Boolean`], or [`List`],
	/// the [`to_integer`], [`to_text`], [`to_boolean`], and [`to_list`] methods do additional logic
	/// for known types.
	///
	/// [`to_integer`]: Self::to_integer
	/// [`to_text`]: Self::to_text
	/// [`to_boolean`]: Self::to_boolean
	/// [`to_list`]: Self::to_list
	pub fn convert<C: NamedType + AttrConversionDefined + Convertible>(self) -> Result<C> {
		if let Some(this) = self.downcast::<C>() {
			return Ok(this);
		}

		let convert = self.call_attr(C::ATTR_NAME, Args::default())?;

		convert
			.downcast::<C>()
			.ok_or_else(|| ErrorKind::ConversionFailed { object: convert, into: C::TYPENAME }.into())
	}

	/// Checks to see if `self` is a `T`.
	#[must_use]
	pub fn is_a<T: Convertible>(self) -> bool {
		T::is_a(self)
	}

	/// Converts `self` to `T`, if `self` is a `T`.
	#[must_use]
	pub fn downcast<T: Convertible>(self) -> Option<T> {
		T::downcast(self).map(T::get)
	}

	/// Attempts to [`downcast`](Self::downcast) `self` to a `T`, returning an `Err` if it cant.
	pub fn try_downcast<T: Convertible + NamedType>(self) -> Result<T> {
		self.downcast().ok_or_else(|| {
			ErrorKind::InvalidTypeGiven { expected: T::TYPENAME, given: self.typename() }.into()
		})
	}

	/// Attempts to hash `self`, returning an `Err` if unable to.
	pub fn try_hash(self) -> Result<u64> {
		if self.is_allocated() {
			if let Some(text) = self.downcast::<Gc<Text>>() {
				Ok(text.as_ref()?.fast_hash())
			} else {
				self
					.call_attr(Intern::hash, Args::default())?
					.try_downcast::<Integer>()
					.map(|n| n.get() as u64)
			}
		} else {
			// this can also be modified, but that's a future thing
			Ok(self.bits())
		}
	}

	/// Attempts to comapre `self` and `rhs`, returning an `Err` if unable to.
	pub fn try_eq(self, rhs: Self) -> Result<bool> {
		if self.is_identical(rhs) {
			return Ok(true);
		}

		if self.is_allocated() {
			if let (Some(lhs), Some(rhs)) = (self.downcast::<Gc<Text>>(), rhs.downcast::<Gc<Text>>()) {
				Ok(*lhs.as_ref()? == *rhs.as_ref()?)
			} else {
				self.call_attr(Intern::op_eql, Args::new(&[rhs], &[]))?.try_downcast()
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

impl Debug for Value {
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
		} else if let Some(l) = self.downcast::<Gc<BigNum>>() {
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
			$lit.to_value()
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
			greeting.downcast::<Gc<Text>>().unwrap().as_ref().unwrap().as_str()
		);
	}

	#[test]
	fn test_call_attrs() {
		let greeting = value!("Hello, world");
		greeting.call_attr(Intern::concat, args!["!"]).unwrap();
		greeting.call_attr(Intern::concat, args![greeting]).unwrap();

		assert_eq!(
			"Hello, world!Hello, world!",
			greeting.downcast::<Gc<Text>>().unwrap().as_ref().unwrap().as_str()
		);

		assert!(greeting
			.call_attr(Intern::op_eql, args![greeting])
			.unwrap()
			.downcast::<Boolean>()
			.unwrap());

		let five = value!(5);
		let twelve = value!(12);
		assert_eq!(
			Integer::new(17).unwrap(),
			five.call_attr(Intern::op_add, args![twelve]).unwrap().downcast::<Integer>().unwrap()
		);

		let ff = value!(255).call_attr(Intern::to_text, args!["base" => 16]).unwrap();
		assert_eq!("ff", ff.downcast::<Gc<Text>>().unwrap().as_ref().unwrap().as_str());
	}
}
