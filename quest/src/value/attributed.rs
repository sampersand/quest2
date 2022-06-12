use crate::value::base::{
	Attribute, AttributesMut, AttributesRef, IntoParent, ParentsMut, ParentsRef,
};
// use crate::vm::Args;
use crate::{ErrorKind, Result, ToValue, Value};

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

	fn get_unbound_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		self.get_unbound_attr_checked(attr, &mut Vec::new())
	}

	fn get_attr<A: Attribute>(&self, attr: A) -> Result<Option<Value>> {
		self.get_unbound_attr(attr)
	}

	fn has_attr<A: Attribute>(&self, attr: A) -> Result<bool> {
		self.get_unbound_attr(attr).map(|x| x.is_some())
	}

	fn try_get_attr<A: Attribute>(&self, attr: A) -> Result<Value>
	where
		Self: Clone + ToValue,
	{
		self.get_attr(attr)?.ok_or_else(|| {
			ErrorKind::UnknownAttribute { object: self.clone().to_value(), attribute: attr.to_value() }
				.into()
		})
	}
}

pub trait AttributedMut: Attributed {
	fn get_unbound_attr_mut<A: Attribute>(&mut self, attr: A) -> Result<&mut Value>;
	fn set_attr<A: Attribute>(&mut self, attr: A, value: Value) -> Result<()>;
	fn del_attr<A: Attribute>(&mut self, attr: A) -> Result<Option<Value>>;
}

pub trait HasAttributes {
	fn attributes(&self) -> AttributesRef<'_>;
	fn attributes_mut(&mut self) -> AttributesMut<'_>;
}

pub trait HasParents {
	fn parents(&self) -> ParentsRef<'_>;
	fn parents_mut(&mut self) -> ParentsMut<'_>;
	fn set_parents<T: IntoParent>(&mut self, parents: T) {
		self.parents_mut().set(parents);
	}
}
