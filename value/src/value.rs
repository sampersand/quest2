use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU64;

#[repr(transparent)]
pub struct Value<T: ?Sized>(NonZeroU64, PhantomData<T>);

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
}

pub enum Any {}
pub type AnyValue = Value<Any>;

impl AnyValue {
    #[inline]
    pub fn is_a<T: crate::Convertible>(self) -> bool {
        T::is_a(self)
    }

    #[inline]
    pub fn downcast<T: crate::Convertible>(self) -> Option<Value<T>> {
        T::downcast(self)
    }
}

impl<T: crate::Convertible> Debug for Value<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Value(")?;
        Debug::fmt(&T::get(*self), f)?;
        f.write_str(")")
    }
}

impl Debug for AnyValue {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        use crate::ty::*;
        use crate::Gc;

        if let Some(i) = self.downcast::<Integer>() {
            Debug::fmt(&i, fmt)
        } else if let Some(f) = self.downcast::<Float>() {
            Debug::fmt(&f, fmt)
        } else if let Some(b) = self.downcast::<Boolean>() {
            Debug::fmt(&b, fmt)
        } else if let Some(n) = self.downcast::<Null>() {
            Debug::fmt(&n, fmt)
        } else if let Some(t) = self.downcast::<Gc<Text>>() {
            Debug::fmt(&t, fmt)
        } else if let Some(i) = self.downcast::<Gc<Integer>>() {
            Debug::fmt(&i, fmt)
        } else if let Some(f) = self.downcast::<Gc<Float>>() {
            Debug::fmt(&f, fmt)
        } else if let Some(b) = self.downcast::<Gc<Boolean>>() {
            Debug::fmt(&b, fmt)
        } else if let Some(n) = self.downcast::<Gc<Null>>() {
            Debug::fmt(&n, fmt)
        } else {
            write!(
                fmt,
                "Value(<unknown:{:p}>)",
                self.0.get() as usize as *const ()
            )
        }
    }
}
