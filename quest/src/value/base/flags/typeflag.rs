use super::Flags;
use crate::value::ty;
use crate::vm;

const fn offset(count: u32) -> u32 {
	assert!(count <= 31);
	count << Flags::TYPE_FLAG_BITSHIFT
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum TypeFlag {
	BigNum = offset(0),
	BoundFn = offset(1),
	Class = offset(2),
	List = offset(3),
	Object = offset(4),
	Scope = offset(5),
	Text = offset(6),
	Wrap = offset(7),
	Frame = offset(8),
	Block = offset(9),

	// TODO: should these be their own types, or inherit from some "base object" type?
	Iterator = offset(11),
	Callable = offset(10),
	Pristine = offset(12),
	ScopeClass = offset(13),
	BoundFnClass = offset(14),
	ThreadClass = offset(15),
}

impl TypeFlag {
	// SAFETY: `bits` needs to be a valid `TypeFlag` bit representation.
	pub unsafe fn from_bits_unchecked(bits: u32) -> Self {
		debug_assert!(Self::is_valid(bits));

		std::mem::transmute::<u32, Self>(bits)
	}

	fn is_valid(inp: u32) -> bool {
		match inp {
			_ if inp == Self::BigNum as u32 => true,
			_ if inp == Self::BoundFn as u32 => true,
			_ if inp == Self::Class as u32 => true,
			_ if inp == Self::List as u32 => true,
			_ if inp == Self::Object as u32 => true,
			_ if inp == Self::Scope as u32 => true,
			_ if inp == Self::Text as u32 => true,
			_ if inp == Self::Wrap as u32 => true,
			_ if inp == Self::Frame as u32 => true,
			_ if inp == Self::Block as u32 => true,
			_ if inp == Self::Callable as u32 => true,
			_ if inp == Self::Iterator as u32 => true,
			_ if inp == Self::Pristine as u32 => true,
			_ if inp == Self::ScopeClass as u32 => true,
			_ => false,
		}
	}
}

// Indicates a type has a `TypeFlag` associated with it.
//
// SAFETY: you must guarantee that the type is unique to this type, and does not collide with other
// types.
pub unsafe trait HasTypeFlag {
	const TYPE_FLAG: TypeFlag;
}

unsafe impl HasTypeFlag for ty::BigNum {
	const TYPE_FLAG: TypeFlag = TypeFlag::BigNum;
}

unsafe impl HasTypeFlag for ty::BoundFn {
	const TYPE_FLAG: TypeFlag = TypeFlag::BoundFn;
}

unsafe impl HasTypeFlag for ty::Class {
	const TYPE_FLAG: TypeFlag = TypeFlag::Class;
}

unsafe impl HasTypeFlag for ty::List {
	const TYPE_FLAG: TypeFlag = TypeFlag::List;
}

unsafe impl HasTypeFlag for ty::Object {
	const TYPE_FLAG: TypeFlag = TypeFlag::Object;
}

unsafe impl HasTypeFlag for ty::Scope {
	const TYPE_FLAG: TypeFlag = TypeFlag::Scope;
}

unsafe impl HasTypeFlag for ty::Text {
	const TYPE_FLAG: TypeFlag = TypeFlag::Text;
}

unsafe impl<T> HasTypeFlag for ty::Wrap<T> {
	const TYPE_FLAG: TypeFlag = TypeFlag::Wrap;
}

unsafe impl HasTypeFlag for vm::Block {
	const TYPE_FLAG: TypeFlag = TypeFlag::Block;
}

unsafe impl HasTypeFlag for vm::Frame {
	const TYPE_FLAG: TypeFlag = TypeFlag::Frame;
}

unsafe impl HasTypeFlag for ty::Callable {
	const TYPE_FLAG: TypeFlag = TypeFlag::Callable;
}

unsafe impl HasTypeFlag for ty::Iterator {
	const TYPE_FLAG: TypeFlag = TypeFlag::Iterator;
}

unsafe impl HasTypeFlag for ty::Pristine {
	const TYPE_FLAG: TypeFlag = TypeFlag::Pristine;
}

unsafe impl HasTypeFlag for ty::scope::ScopeClass {
	const TYPE_FLAG: TypeFlag = TypeFlag::ScopeClass;
}

unsafe impl HasTypeFlag for ty::boundfn::BoundFnClass {
	const TYPE_FLAG: TypeFlag = TypeFlag::BoundFnClass;
}
