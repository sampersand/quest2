use crate::vm::SourceLocation;
use crate::AnyValue;

pub const MAX_ARGUMENTS_FOR_SIMPLE_CALL: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
	NoOp,
	Debug,
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
}

impl Opcode {
	pub fn unary_from_symbol(symbol: &str) -> Option<Self> {
		match symbol {
			"!" => Some(Self::Not),
			"-" => Some(Self::Negate),
			_ => None,
		}
	}

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
}
