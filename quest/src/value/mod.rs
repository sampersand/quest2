//! Types relating to [`Value`] and its representation in Quest.

#[macro_use]
pub mod ty;

mod attributed;
pub mod base;
pub mod gc;
mod value;

pub use attributed::{
	Attributed, AttributedMut, Callable, HasAttributes, HasFlags, HasParents, TryAttributed,
};
pub use gc::Gc;
pub use value::Value;

/// A trait that indicates that a type has a default parent associated with it.
///
/// This is used both when creating new [`Base`](base::Base)s and interacting with non-allocated
/// [`Value`]s.
pub trait HasDefaultParent {
	/// Gets the default parent.
	///
	/// The returned value should not change between calls.
	fn parent() -> Value;
}

/// The name of a type (eg `"Integer"`).
///
/// See also [`NamedType`].
pub type Typename = &'static str;

/// A Trait representing that a type has a name within Quest.
pub trait NamedType {
	/// The name of the type.
	const TYPENAME: Typename;
}

/// A trait that indicates a type can be converted to and from a [`Value`].
///
/// # Safety
/// You must ensure that [`is_a`](Convertible::is_a) returns `true` only for [`Value`]s of `Self`
/// type, and nothing else. Additionally, you must ensure that no other type shares the same
/// representation as `Self`.
///
/// This trait might become sealed at a later point.
pub unsafe trait Convertible: Into<Value<Self>> {
	/// Checks to see if the generic [`Value`] is actually a `Value<Self>`.
	fn is_a(value: Value) -> bool;

	/// Attempts to downcast `value` to a `Value<Self>`, returning `None` if it's not possible.
	#[must_use]
	fn downcast(value: Value) -> Option<Value<Self>> {
		// SAFETY: the implementation guarantees that `is_a` will return true iff `value` is
		// exclusively a `Value<Self>`. As such, the conversion is warrented
		Self::is_a(value).then(|| unsafe { std::mem::transmute(value) })
	}

	/// Unwraps the `value`, returning the enclosed `Self` type.
	fn get(value: Value<Self>) -> Self;
}

/// A trait that indicates a type can be freely converted to a [`Value`].
///
/// Unlike [`Convertible`], this doesn't exclusively need to be implemented on types that have an
/// immediate representation within Quest: For example, [`String`] has an implementation which
/// creates a new [`Text`](ty::Text) for it, and then converts the `Text` to a [`Value`].
pub trait ToValue {
	/// Perform the to value conversion.
	#[must_use]
	fn to_value(self) -> Value;
}

impl<T: Convertible> ToValue for T {
	fn to_value(self) -> Value {
		self.into().to_value()
	}
}

impl ToValue for std::cmp::Ordering {
	fn to_value(self) -> Value {
		match self {
			Self::Less => (-1).to_value(),
			Self::Equal => 0.to_value(),
			Self::Greater => 1.to_value(),
		}
	}
}
