#[repr(u8)]
enum Variable {
	Yes = 1 << 7,
	No = 0,
}
#[repr(u8)]
enum Interned {
	Yes = 1 << 6,
	No = 0,
}

// opcode format: VI_CCCC_AA
// `V` is whether it takes a variable amount of arguments
// `I` is whether it takes an interned variable after all positional (and variable) arguments
// `C` is its count
// `A` is arity
const fn opcode_fmt(variable: Variable, interned: Interned, fixed_arity: u8, count: u8) -> u8 {
	assert!(fixed_arity <= 0b11);
	assert!(count <= 0b1111);

	variable as u8 | interned as u8 | (count << 2) | fixed_arity
}

/// The list of opcodes the Quest interpreter supports.
///
/// The numeric representations were very carefully chosen: Divide the number by `0x20` (taking the
/// absolute value first) yields the amount of fixed locals the opcode takes.
///
/// If the opcode is negative, it indicates that the opcode additionally accepts a variable amount
/// of locals (but at most [`NUM_ARGUMENT_REGISTERS`](super::NUM_ARGUMENT_REGISTERS).
/// (For opcodes that expect more than `NUM_ARGUMENT_REGISTERS`, they're positive).
///
/// All opcodes take a destination operand as their first argument, including ones that don't
/// _really_ need a destination (eg [`IndexAssign`](Self::IndexAssign)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum Opcode {
	/// `CreateList(dst, count, ...)` Create a list of size `count` of trailing locals and stores it
	/// into `dst`. (For short lists, use [`CreateListSimple`](Self::CreateListSimple), as it uses an
	/// internal buffer).
	CreateList = opcode_fmt(Variable::No, Interned::No, 0, 0),

	/// `CreateListSimple(dst, count, ...)` Create a list of size `count` of trailing locals and
	/// stores it into `dst`. (For longer lists, use [`CreateList`](Self::CreateList))
	CreateListSimple = opcode_fmt(Variable::Yes, Interned::No, 0, 0),

	/// `ConstLoad(dst, count)` Loads the constant at `count` into `dst`.
	ConstLoad = opcode_fmt(Variable::No, Interned::No, 0, 1),

	/// `LoadImmediate(dst, <8 bytes>)` interprets the following 8 bytes as a [`Value`].
	LoadImmediate = opcode_fmt(Variable::No, Interned::No, 0, 2),

	/// `LoadImmediate(dst, <1 byte>)` interprets the following `i8` as a [`Value`], sign-extending.
	LoadSmallImmediate = opcode_fmt(Variable::No, Interned::No, 0, 3),

	/// `LoadBlock(dst, <8 bytes>)` interprets the following 8 bytes as a [`Gc<Block>`], adding the
	/// currently executing frame as a parent.
	LoadBlock = opcode_fmt(Variable::No, Interned::No, 0, 4),

	/// `Stackframe(dst, count)` Gets the `count`th stackframe. Can be negative.
	Stackframe = opcode_fmt(Variable::No, Interned::No, 0, 5),

	/* ARITY ONE */
	/// `Mov(dst, src)` Copies `src` into `dst`.
	Mov = opcode_fmt(Variable::No, Interned::No, 1, 0),

	/// `Call(dst, fn, ???)` todo comeback later
	Call = opcode_fmt(Variable::No, Interned::No, 1, 1),

	/// `CallSimple(dst, fn, count, ...)` Calls `fn` with `count` positional arguments.
	CallSimple = opcode_fmt(Variable::Yes, Interned::No, 1, 0),

	/// `Index(dst, ary, count, ...)` Indexes into `ary` with `count` arguments.
	Index = opcode_fmt(Variable::Yes, Interned::No, 1, 1),

	/// `IndexAssign(dst, ary, count, ...)` Index-assigns `ary` with `count` arguments.
	/// Note that this doesn't have a separate "value to store" operand, as that's simply the last
	/// positional argument.
	IndexAssign = opcode_fmt(Variable::Yes, Interned::No, 1, 2),

	/// `Not(dst, src)` Logically negates `src`, pushing it into `dst`.
	Not = opcode_fmt(Variable::No, Interned::No, 1, 2),

	/// `Negate(dst, src)` Numerically negates `src`, pushing it into `dst`.
	Negate = opcode_fmt(Variable::No, Interned::No, 1, 3),

	/** MATH OPS **/

	/// `Add(dst, lhs, rhs)` Sets `dst` to the result of adding `lhs + rhs`.
	Add = opcode_fmt(Variable::No, Interned::No, 2, 0),

	/// `Subtract(dst, lhs, rhs)` Sets `dst` to the result of `lhs - rhs`.
	Subtract = opcode_fmt(Variable::No, Interned::No, 2, 1),

	/// `Multiply(dst, lhs, rhs)` Sets `dst` to the result of `lhs * rhs`.
	Multiply = opcode_fmt(Variable::No, Interned::No, 2, 2),

	/// `Divide(dst, lhs, rhs)` Sets `dst` to the result of `lhs / rhs`.
	Divide = opcode_fmt(Variable::No, Interned::No, 2, 3),

	/// `Modulo(dst, lhs, rhs)` Sets `dst` to the result of `lhs % rhs`.
	Modulo = opcode_fmt(Variable::No, Interned::No, 2, 4),

	/// `Equal(dst, lhs, rhs)` Sets `dst` to the result of `lhs == rhs`.
	Equal = opcode_fmt(Variable::No, Interned::No, 2, 5),

	/// `NotEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs != rhs`.
	NotEqual = opcode_fmt(Variable::No, Interned::No, 2, 6),

	/// `LessThan(dst, lhs, rhs)` Sets `dst` to the result of `lhs < rhs`.
	LessThan = opcode_fmt(Variable::No, Interned::No, 2, 7),

	/// `GreaterThan(dst, lhs, rhs)` Sets `dst` to the result of `lhs > rhs`.
	GreaterThan = opcode_fmt(Variable::No, Interned::No, 2, 8),

	/// `LessEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs <= rhs`.
	LessEqual = opcode_fmt(Variable::No, Interned::No, 2, 9),

	/// `GreaterEqual(dst, lhs, rhs)` Sets `dst` to the result of `lhs >= rhs`.
	GreaterEqual = opcode_fmt(Variable::No, Interned::No, 2, 10),

	/// `Compare(dst, lhs, <number 1>, rhs)` Sets `dst` to the result of `lhs <=> rhs`.
	/// Note that this uses a variable amount of arguments, as there's not enough room in the
	/// non-variable argument count to support it. However, the arity will always be one.
	Compare = opcode_fmt(Variable::Yes, Interned::No, 1, 11),

	/// `Power(dst, lhs, <number 1>, rhs)` Sets `dst` to the result of `lhs ** rhs`.
	/// Note that this uses a variable amount of arguments, as there's not enough room in the
	/// non-variable argument count to support it. However, the arity will always be one.
	Power = opcode_fmt(Variable::Yes, Interned::No, 1, 12),

	/* ATTRIBUTES */
	/// `GetAttr(dst, obj, attr)` Gets the attribute `attr` on `obj`, storing the result into `dst`.
	GetAttr = opcode_fmt(Variable::No, Interned::No, 2, 11),

	/// `GetUnboundAttr(dst, obj, attr)` Gets the unbound attribute `attr` on `obj`, storing the
	/// result into `dst`.
	GetUnboundAttr = opcode_fmt(Variable::No, Interned::No, 2, 12),

	/// `HasAttr(dst, obj, attr)` Checks to see if the attribute `attr` on `obj`, storing the result
	/// into `dst`.
	HasAttr = opcode_fmt(Variable::No, Interned::No, 2, 13),

	/// `SetAttr(dst, value, attr[, obj])` Sets the attribute `attr` on `obj` to `value`.
	/// Note that `obj` is not actually read as part of the bytecode, as we may be assigning to an
	/// [`Integer`](crate::value::ty::Integer) (or another type), which means we may need to mutably
	/// get the attribute.
	SetAttr = opcode_fmt(Variable::No, Interned::No, 2, 14),

	/// `DelAttr(dst, obj, <number 1>, attr)` Deletes the attribute `rhs` from `lhs`.
	/// Note that this uses a variable amount of arguments, as there's not enough room in the
	/// non-variable argument count to support it. However, the arity will always be one.
	DelAttr = opcode_fmt(Variable::Yes, Interned::No, 1, 5),

	/// `CallAttr(dst, fn, ???)` todo comeback later
	CallAttr = opcode_fmt(Variable::No, Interned::No, 2, 15),

	/// `CallAttrSimple(dst, obj, attr, count, ...)` Calls `obj`'s attribute `attr` with `count`
	/// positional arguments, storing the result into `dst`.
	CallAttrSimple = opcode_fmt(Variable::Yes, Interned::No, 2, 3),

	/* INTERNED ATTRIBUTES */
	/// `GetAttr(dst, obj, <intern attr>)`
	GetAttrIntern = opcode_fmt(Variable::No, Interned::Yes, 1, 11),

	/// `GetUnboundAttrIntern(dst, obj, <intern attr>)`
	GetUnboundAttrIntern = opcode_fmt(Variable::No, Interned::Yes, 1, 12),

	/// `HasAttrIntern(dst, obj, <intern attr>)`
	HasAttrIntern = opcode_fmt(Variable::No, Interned::Yes, 1, 13),

	/// `SetAttrIntern(dst, value, <intern attr>[, obj])`. See `SetAttr` for details
	SetAttrIntern = opcode_fmt(Variable::No, Interned::Yes, 1, 14),

	// <TODO>
	CallAttrIntern = opcode_fmt(Variable::Yes, Interned::Yes, 1, 15),

	/// `CallAttrSimpleIntern(dst, obj, <intern>, count, ...)` Calls `obj`'s attribute `attr` with
	/// `count` positional arguments, storing the result into `dst`.
	CallAttrSimpleIntern = opcode_fmt(Variable::Yes, Interned::Yes, 1, 3),

	/// `DelAttrIntern(dst, obj, <intern>)` Deletes `<intern>` from `obj`. If you want to dynamically
	/// delete variables, you must use a `CallAttrSimpleIntern` with a `__del_attr__` argument.
	DelAttrIntern = opcode_fmt(Variable::No, Interned::Yes, 1, 0),
}

impl Opcode {
	/// Gets the arity of `self`.
	#[inline]
	pub const fn fixed_arity(self) -> usize {
		(self as u8 & 0b11) as usize
	}

	#[inline]
	pub const fn takes_intern(self) -> bool {
		((self as u8) & 0b01_0000_00) != 0
	}

	/// Gets whether `self` takes a variable amount of arguments.
	#[inline]
	pub const fn is_variable_simple(self) -> bool {
		(self as i8).is_negative()
	}

	/// <DOC: TODO>
	#[inline]
	pub const fn count_within_arity(self) -> usize {
		((self as u8 & 0b00_1111_00) >> 2) as usize
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

	pub const fn from_byte(byte: u8) -> Option<Self> {
		if Self::verify_is_valid(byte) {
			Some(unsafe { std::mem::transmute(byte) })
		} else {
			None
		}
	}

	/// Returns whether `byte` actually corresponds to a valid [`Opcode`].
	#[allow(clippy::cognitive_complexity)]
	pub const fn verify_is_valid(byte: u8) -> bool {
		match byte {
			_ if byte == Self::CreateList as u8 => true,
			_ if byte == Self::CreateListSimple as u8 => true,

			_ if byte == Self::Mov as u8 => true,
			_ if byte == Self::Call as u8 => true,
			_ if byte == Self::CallSimple as u8 => true,
			_ if byte == Self::ConstLoad as u8 => true,
			_ if byte == Self::LoadImmediate as u8 => true,
			_ if byte == Self::LoadSmallImmediate as u8 => true,
			_ if byte == Self::LoadBlock as u8 => true,
			_ if byte == Self::Stackframe as u8 => true,

			_ if byte == Self::GetAttr as u8 => true,
			_ if byte == Self::GetUnboundAttr as u8 => true,
			_ if byte == Self::HasAttr as u8 => true,
			_ if byte == Self::SetAttr as u8 => true,
			_ if byte == Self::DelAttr as u8 => true,
			_ if byte == Self::CallAttr as u8 => true,
			_ if byte == Self::CallAttrSimple as u8 => true,
			_ if byte == Self::GetAttrIntern as u8 => true,
			_ if byte == Self::GetUnboundAttrIntern as u8 => true,
			_ if byte == Self::HasAttrIntern as u8 => true,
			_ if byte == Self::SetAttrIntern as u8 => true,
			_ if byte == Self::DelAttrIntern as u8 => true,
			_ if byte == Self::CallAttrIntern as u8 => true,
			_ if byte == Self::CallAttrSimpleIntern as u8 => true,

			_ if byte == Self::Not as u8 => true,
			_ if byte == Self::Negate as u8 => true,
			_ if byte == Self::Equal as u8 => true,
			_ if byte == Self::NotEqual as u8 => true,
			_ if byte == Self::LessThan as u8 => true,
			_ if byte == Self::GreaterThan as u8 => true,
			_ if byte == Self::LessEqual as u8 => true,
			_ if byte == Self::GreaterEqual as u8 => true,
			_ if byte == Self::Compare as u8 => true,
			_ if byte == Self::Add as u8 => true,
			_ if byte == Self::Subtract as u8 => true,
			_ if byte == Self::Multiply as u8 => true,
			_ if byte == Self::Divide as u8 => true,
			_ if byte == Self::Modulo as u8 => true,
			_ if byte == Self::Power as u8 => true,
			_ if byte == Self::Index as u8 => true,
			_ if byte == Self::IndexAssign as u8 => true,
			_ => false,
		}
	}
}
