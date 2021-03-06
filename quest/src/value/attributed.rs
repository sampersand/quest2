use crate::value::base::{
	Attribute, AttributesMut, AttributesRef, Flags, IntoParent, ParentsMut, ParentsRef,
};
use crate::value::{ty, Gc};
use crate::vm::Args;
use crate::{ErrorKind, Intern, Result, ToValue, Value};

pub trait Attributed {
	/// Get the unbound attribute `attr`, with a list of checked parents, `None` if it doesnt exist.
	///
	/// The `checked` parameter allows us to keep track of which parents have already been checked,
	/// so as to prevent checking the same parents more than once.
	fn get_unbound_attr_checked<A: Attribute>(
		&self,
		attr: A,
		checked: &mut Vec<Value>,
	) -> Result<Option<Value>>;

	/// Get the unbound attribute `attr`, returning `None` if it doesnt exist.
	fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		self.get_unbound_attr_checked(attr, &mut Vec::new())
	}

	/// Checks to see if `self` has the attribute `attr`.
	fn has_attr<A: Attribute>(&self, attr: A) -> Result<bool> {
		self.get_unbound_attr(attr).map(|x| x.is_some())
	}
}

pub trait TryAttributed: Attributed + Copy + ToValue {
	/// Attempts to get the unbound attribute `attr`, returning `Err` if it doesn't exist.
	fn try_get_unbound_attr<A: Attribute>(self, attr: A) -> Result<Value> {
		self.get_unbound_attr(attr)?.ok_or_else(|| {
			ErrorKind::UnknownAttribute { object: self.to_value(), attribute: attr.to_value() }.into()
		})
	}

	/// Get the attribute `attr`, returning `None` if it doesnt exist.
	///
	/// For attributes which have [`Intern::op_call`] defined on them, this will create a new
	/// [`BoundFn`]. For all other types, it just returns the attribute itself.
	fn get_attr<A: Attribute>(self, attr: A) -> Result<Option<Value>> {
		let value = if let Some(value) = self.get_unbound_attr(attr)? {
			value
		} else {
			return Ok(None);
		};

		let is_callable = value.is_a::<ty::RustFn>()
			|| value.is_a::<Gc<crate::vm::Block>>()
			|| value.is_a::<Gc<ty::BoundFn>>()
			|| value.has_attr(Intern::op_call)?;

		// If the value is callable, wrap it in a bound fn. Short circuit for common ones.
		if is_callable {
			return Ok(Some(ty::BoundFn::new(self.to_value(), value).to_value()));
		}

		Ok(Some(value))
	}

	/// Attempts to get the attribute `attr`, returning `Err` if it doesn't exist.
	fn try_get_attr<A: Attribute>(self, attr: A) -> Result<Value> {
		self.get_attr(attr)?.ok_or_else(|| {
			ErrorKind::UnknownAttribute { object: self.to_value(), attribute: attr.to_value() }.into()
		})
	}
}

impl<T: Attributed + Copy + ToValue> TryAttributed for T {}

pub trait Callable {
	/// Calls `self` with the given `args`.
	fn call(self, args: Args<'_>) -> Result<Value>;
}

pub trait AttributedMut {
	/// Gets mutable access to the attribute `attr`, creating it if on `Self` if it doesn't exist.
	///
	/// This doesn't have an "checked" variant, as only attributes are looked at.
	fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value>;

	/// Sets the attribute `attr` on `self` to `value`.
	///
	/// Note that this takes a mutable reference to `self` in case `self` is not an allocated type:
	/// If it isn't, `self` will be replaced with an allocated version, with the attribute set on
	/// that type.
	fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()>;

	/// Deletes the attribute `attr` from `self`, returning whatever was there before.
	///
	/// Note that unallocated types don't actually have attributes defined on them, so they always
	/// will return `Ok(None)`
	fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>>;
}

pub trait HasFlags {
	/// Gets the flags associated with the current object.
	#[must_use]
	fn flags(&self) -> &Flags;

	/// Sets flags in the header's [`Flags`].
	///
	/// If you want to _access_ the flags, i.e. don't want to set them, then use [`flags`](
	/// Self::flags). See [`Flags`] for more details on custom flags in general.
	///
	//	/// # Examples
	//	/// ```text
	//	/// // this is `text` because i plan on overhauling the builder.
	//	/// # use quest::value::base::{Base, Flags};
	//	/// let mut builder = Base::<()>::builder();
	//	///
	//	/// // Set custom flags, if you attribute meanings to them.
	//	/// const FLAG_IS_SUPER_DUPER_COOL: u32 = Flags::USER0;
	//	/// builder.insert_user_flags(FLAG_IS_SUPER_DUPER_COOL);
	//	///
	//	/// // SAFETY: Since `builder` was zero-initialized to a ZST, we didn't have to do anything.
	//	/// let base = unsafe { builder.finish() };
	//	///
	//	/// assert!(
	//	///     base.as_ref().expect("we hold the only reference")
	//	///         .flags()
	//	///         .contains(FLAG_IS_SUPER_DUPER_COOL)
	//	/// );
	//	/// ```
	fn insert_user_flags(&self, flag: u32) {
		self.flags().insert_user(flag);
	}
}

pub trait HasAttributes {
	/// Gets an immutable reference to `self`'s attributes.
	fn attributes(&self) -> AttributesRef<'_>;

	/// Gets a mutable reference to `self`'s attributes.
	fn attributes_mut(&mut self) -> AttributesMut<'_>;
}

pub trait HasParents {
	/// Gets an immutable reference to `self`'s parents.
	fn parents(&self) -> ParentsRef<'_>;

	/// Gets a mutable reference to `self`'s parents.
	fn parents_mut(&mut self) -> ParentsMut<'_>;

	/// Replaces `self`'s parents with `new_parents`
	fn set_parents<T: IntoParent>(&mut self, new_parents: T) {
		self.parents_mut().set(new_parents);
	}

	/// Gets the list of parents associated with `self`, converting non-list parents into one.
	#[must_use]
	fn parents_list(&mut self) -> crate::value::Gc<crate::value::ty::List> {
		self.parents_mut().as_list()
	}
}
