use crate::vm::SourceLocation;
use crate::AnyValue;

type Local = usize;
type Offset = isize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
	NoOp,
	Debug,

	Mov,
	// Jmp,
	// JmpFalse,
	// JmpTrue,
	Call,
	CallSimple,
	// Return,
	ConstLoad,
	CurrentFrame,
	GetAttr,
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
