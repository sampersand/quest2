/// The list of opcodes the Quest interpreter supports.
///
/// The numeric representations were very carefully chosen: Divide the number by `0x20` (taking the
/// absolute value first) yields the amount of fixed locals the opcode takes.
///
/// If the opcode is negative, it indicates that the opcode additionally accepts a variable amount
/// of locals (but at most [`MAX_ARGUMENTS_FOR_SIMPLE_CALL`](super::MAX_ARGUMENTS_FOR_SIMPLE_CALL).
/// (For opcodes that expect more than `MAX_ARGUMENTS_FOR_SIMPLE_CALL`, they're positive).
///
/// All opcodes take a destination operand as their first argument, including ones that don't
/// _really_ need a destination (eg [`SetAttr`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum Opcode {
	/// `CreateList(dst, count, ...)` Create a list of size `count` of trailing locals and stores it
	/// into `dst`. (For short lists, use [`CreateListShort`], as it uses an internal buffer). 
	CreateList = 0x00,
	/// `CreateListShort(dst, count, ...)` Create a list of size `count` of trailing locals and
	/// stores it into `dst`. (For longer lists, use [`CreateList`])
	CreateListShort = -0x01i8 as u8,
	/// `ConstLoad(dst, count)` Loads the constant at `count` into `dst`.
	ConstLoad = 0x01,
	/// `Stackframe(dst, count)` Gets the `count`th stackframe. Can be negative.
	Stackframe = 0x02,

	/// `Mov(dst, src)` Copies `src` into `dst`.
	Mov = 0x20,
	/// `Call(dst, fn, ???)` todo comeback later
	Call = 0x21,
	/// `CallSimple(dst, fn, count, ...)` Calls `fn` with `count` positional arguments.
	CallSimple = -0x23i8 as u8,
	/// `Index(dst, ary, count, ...)` Indexes into `ary` with `count` arguments.
	Index = -0x21i8 as u8,
	/// `IndexAssign(dst, ary, count, ...)` Index-assigns `ary` with `count` arguments.
	/// Note that this doesn't have a separate "value to store" operand, as that's simply the last
	/// positional argument.
	IndexAssign = -0x22i8 as u8,
	/// `Not(dst, src)` Logically negates `src`, pushing it into `dst`.
	Not = 0x22,
	/// `Negate(dst, src)` Numerically negates `src`, pushing it into `dst`.
	Negate = 0x23,

	/// `GetAttr(dst, obj, attr)` Gets the attribute `attr` on `obj`, storing the result into `dst`.
	GetAttr = 0x40,
	/// `GetUnboundAttr(dst, obj, attr)` Gets the unbound attribute `attr` on `obj`, storing the
	/// result into `dst`.
	GetUnboundAttr = 0x41,
	/// `HasAttr(dst, obj, attr)` Checks to see if the attribute `attr` on `obj`, storing the result
	/// into `dst`.
	HasAttr = 0x42,
	/// `SetAttr(dst, attr, value[, obj])` Sets the attribute `attr` on `obj` to `value`.
	/// Note that `obj` is not actually read as part of the bytecode, as we may be assigning to an
	/// [`Integer`](crate::value::ty::Integer) (or another type), which means we may need to mutably
	/// get the attribute.
	SetAttr = 0x43,
	/// `DelAttr(dst, obj, attr)` Deletes the attribute `attr` from `obj`, storing its previous
	/// value into `dst` (if it doesnt exist, [`Null`](crate::value::ty::Null) is used).
	DelAttr = 0x44,
	/// `CallAttr(dst, fn, ???)` todo comeback later
	CallAttr = 0x45,
	/// `CallAttrSimple(dst, obj, attr, count, ...)` Calls `obj`'s attribute `attr` with `count`
	/// positional arguments, storing the result into `dst`.
	CallAttrSimple = -0x41i8 as u8,
	/// `Add(dst, lhs, rhs)` Sets `dst` to the result of adding `lhs + rhs`.
	Add = 0x46,
	/// `Subtract(dst, lhs, rhs)` Sets `dst` to the result of `lhs - rhs`.
	Subtract = 0x47,
	/// `Multiply(dst, lhs, rhs)` Sets `dst` to the result of `lhs * rhs`.
	Multiply = 0x48,
	/// `Divide(dst, lhs, rhs)` Sets `dst` to the result of `lhs / rhs`.
	Divide = 0x49,
	/// `Modulo(dst, lhs, rhs)` Sets `dst` to the result of `lhs % rhs`.
	Modulo = 0x4a,
	/// `Power(dst, lhs, rhs)` Sets `dst` to the result of `lhs ** rhs`.
	Power = 0x4b,
	/// `Equal(dst, lhs, rhs)` Sets `dst` to the result of `lhs == rhs`.
	Equal = 0x4c,
	/// `NotEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs != rhs`.
	NotEqual = 0x4d,
	/// `LessThan(dst, lhs, rhs)` Sets `dst` to the result of `lhs < rhs`.
	LessThan = 0x4e,
	/// `GreaterThan(dst, lhs, rhs)` Sets `dst` to the result of `lhs > rhs`.
	GreaterThan = 0x4f,
	/// `LessEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs <= rhs`.
	LessEqual = 0x50,
	/// `GreaterEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs >= rhs`.
	GreaterEqual = 0x51,
	/// `Compare(dst, lhs, rhs)` Sets `dst` to the result of `lhs <=> rhs`.
	Compare = 0x52,
}

impl Opcode {
	/// Gets the arity of `self`.
	pub const fn arity(self) -> usize {
		((self as i8) / 0x20).abs() as usize
	}

	/// Gets whether `self` takes a variable amount of arguments.
	pub const fn is_variable_simple(self) -> bool {
		(self as i8).is_negative()
	}

	/// Gets the `Opcode` corresponding to the unary operator `symbol`, if one exists.
	#[must_use]
	pub fn unary_from_symbol(symbol: &str) -> Option<Self> {
		match symbol {
			"!" => Some(Self::Not),
			"-" => Some(Self::Negate),
			_ => None,
		}
	}

	/// Gets the `Opcode` corresponding to the binary operator `symbol`, if one exists.
	#[must_use]
	pub fn binary_from_symbol(symbol: &str) -> Option<Self> {
		// While `()`, `[]`, and `[]=` technically correspond to it, they arent actually
		// able to be used as binary operators from within quest sourcecode.
		match symbol {
			"+" => Some(Self::Add),
			"-" => Some(Self::Subtract),
			"*" => Some(Self::Multiply),
			"/" => Some(Self::Divide),
			"%" => Some(Self::Modulo),
			"**" => Some(Self::Power),

			"==" => Some(Self::Equal),
			"!=" => Some(Self::NotEqual),
			"<" => Some(Self::LessThan),
			">" => Some(Self::GreaterThan),
			"<=" => Some(Self::LessEqual),
			">=" => Some(Self::GreaterEqual),
			"<=>" => Some(Self::Compare),
			_ => None,
		}
	}

	/// Returns whether `byte` actually corresponds to a valid [`Opcode`].
	pub const fn verify_is_valid(byte: u8) -> bool {
		match byte {
			_ if byte == Opcode::CreateList as u8 => true,
			_ if byte == Opcode::CreateListShort as u8 => true,

			_ if byte == Opcode::Mov as u8 => true,
			_ if byte == Opcode::Call as u8 => true,
			_ if byte == Opcode::CallSimple as u8 => true,
			_ if byte == Opcode::ConstLoad as u8 => true,
			_ if byte == Opcode::Stackframe as u8 => true,

			_ if byte == Opcode::GetAttr as u8 => true,
			_ if byte == Opcode::GetUnboundAttr as u8 => true,
			_ if byte == Opcode::HasAttr as u8 => true,
			_ if byte == Opcode::SetAttr as u8 => true,
			_ if byte == Opcode::DelAttr as u8 => true,
			_ if byte == Opcode::CallAttr as u8 => true,
			_ if byte == Opcode::CallAttrSimple as u8 => true,

			_ if byte == Opcode::Not as u8 => true,
			_ if byte == Opcode::Negate as u8 => true,
			_ if byte == Opcode::Equal as u8 => true,
			_ if byte == Opcode::NotEqual as u8 => true,
			_ if byte == Opcode::LessThan as u8 => true,
			_ if byte == Opcode::GreaterThan as u8 => true,
			_ if byte == Opcode::LessEqual as u8 => true,
			_ if byte == Opcode::GreaterEqual as u8 => true,
			_ if byte == Opcode::Compare as u8 => true,
			_ if byte == Opcode::Add as u8 => true,
			_ if byte == Opcode::Subtract as u8 => true,
			_ if byte == Opcode::Multiply as u8 => true,
			_ if byte == Opcode::Divide as u8 => true,
			_ if byte == Opcode::Modulo as u8 => true,
			_ if byte == Opcode::Power as u8 => true,
			_ if byte == Opcode::Index as u8 => true,
			_ if byte == Opcode::IndexAssign as u8 => true,
			_ => false,
		}
	}
}
