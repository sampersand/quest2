use crate::vm::SourceLocation;
use crate::AnyValue;

pub const MAX_ARGUMENTS_FOR_SIMPLE_CALL: usize = 16;
pub(super) const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
#[allow(clippy::manual_non_exhaustive)]
pub enum Opcode {
	CreateList,

	Mov,
	Call,
	CallSimple,
	ConstLoad,
	Stackframe,

	GetAttr,
	GetUnboundAttr,
	HasAttr,
	SetAttr,
	DelAttr,
	CallAttr,
	CallAttrSimple,

	Add,
	Subtract,
	Multuply,
	Divide,
	Modulo,
	Power,

	Not,
	Negate,
	Equal,
	NotEqual,
	LessThan,
	GreaterThan,
	LessEqual,
	GreaterEqual,
	Compare,

	Index,
	IndexAssign,

	#[doc(hidden)]
	__LAST,
}

impl Opcode {
	#[must_use]
	pub fn unary_from_symbol(symbol: &str) -> Option<Self> {
		match symbol {
			"!" => Some(Self::Not),
			"-" => Some(Self::Negate),
			_ => None,
		}
	}

	#[must_use]
	pub fn binary_from_symbol(symbol: &str) -> Option<Self> {
		match symbol {
			"()" => Some(Self::CallSimple),

			"+" => Some(Self::Add),
			"-" => Some(Self::Subtract),
			"*" => Some(Self::Multuply),
			"/" => Some(Self::Divide),
			"%" => Some(Self::Modulo),
			"^" => Some(Self::Power),

			"==" => Some(Self::Equal),
			"!=" => Some(Self::NotEqual),
			"<" => Some(Self::LessThan),
			">" => Some(Self::GreaterThan),
			"<=" => Some(Self::LessEqual),
			">=" => Some(Self::GreaterEqual),
			"<=>" => Some(Self::Compare),

			"[]" => Some(Self::Index),
			"[]=" => Some(Self::IndexAssign),
			_ => None,
		}
	}

	#[must_use]
	pub const fn from_u8(byte: u8) -> Option<Self> {
		if byte < Self::__LAST as u8 {
			Some(unsafe { std::mem::transmute(byte) })
		} else {
			None
		}

		// match byte {
		// 	_ if byte == Opcode::CreateList as u8 => Some(Opcode::CreateList),

		// 	_ if byte == Opcode::Mov as u8 => Some(Opcode::Mov),
		// 	_ if byte == Opcode::Call as u8 => Some(Opcode::Call),
		// 	_ if byte == Opcode::CallSimple as u8 => Some(Opcode::CallSimple),
		// 	_ if byte == Opcode::ConstLoad as u8 => Some(Opcode::ConstLoad),
		// 	_ if byte == Opcode::Stackframe as u8 => Some(Opcode::Stackframe),

		// 	_ if byte == Opcode::GetAttr as u8 => Some(Opcode::GetAttr),
		// 	_ if byte == Opcode::GetUnboundAttr as u8 => Some(Opcode::GetUnboundAttr),
		// 	_ if byte == Opcode::HasAttr as u8 => Some(Opcode::HasAttr),
		// 	_ if byte == Opcode::SetAttr as u8 => Some(Opcode::SetAttr),
		// 	_ if byte == Opcode::DelAttr as u8 => Some(Opcode::DelAttr),
		// 	_ if byte == Opcode::CallAttr as u8 => Some(Opcode::CallAttr),
		// 	_ if byte == Opcode::CallAttrSimple as u8 => Some(Opcode::CallAttrSimple),

		// 	_ if byte == Opcode::Not as u8 => Some(Opcode::Not),
		// 	_ if byte == Opcode::Negate as u8 => Some(Opcode::Negate),
		// 	_ if byte == Opcode::Equal as u8 => Some(Opcode::Equal),
		// 	_ if byte == Opcode::NotEqual as u8 => Some(Opcode::NotEqual),
		// 	_ if byte == Opcode::LessThan as u8 => Some(Opcode::LessThan),
		// 	_ if byte == Opcode::GreaterThan as u8 => Some(Opcode::GreaterThan),
		// 	_ if byte == Opcode::LessEqual as u8 => Some(Opcode::LessEqual),
		// 	_ if byte == Opcode::GreaterEqual as u8 => Some(Opcode::GreaterEqual),
		// 	_ if byte == Opcode::Compare as u8 => Some(Opcode::Compare),
		// 	_ if byte == Opcode::Add as u8 => Some(Opcode::Add),
		// 	_ if byte == Opcode::Subtract as u8 => Some(Opcode::Subtract),
		// 	_ if byte == Opcode::Multuply as u8 => Some(Opcode::Multuply),
		// 	_ if byte == Opcode::Divide as u8 => Some(Opcode::Divide),
		// 	_ if byte == Opcode::Modulo as u8 => Some(Opcode::Modulo),
		// 	_ if byte == Opcode::Power as u8 => Some(Opcode::Power),
		// 	_ if byte == Opcode::Index as u8 => Some(Opcode::Index),
		// 	_ if byte == Opcode::IndexAssign as u8 => Some(Opcode::IndexAssign),
		// 	_ => None,
		// }
	}
}
