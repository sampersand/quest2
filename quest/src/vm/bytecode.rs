pub const MAX_ARGUMENTS_FOR_SIMPLE_CALL: usize = 16;
pub(super) const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum Opcode {
	CreateList =       0x00,
	ConstLoad =        0x01,
	Stackframe =       0x02,
	CreateListShort = -0x01i8 as u8,

	Mov =            0x20,
	Call =           0x21,
	Not =            0x22,
	Negate =         0x23,
	Index = -0x21i8 as u8,
	IndexAssign = -0x22i8 as u8, // the last argument's actually the value to assign
	CallSimple = -0x23i8 as u8,

	GetAttr =        0x40,
	GetUnboundAttr = 0x41,
	HasAttr =        0x42,
	SetAttr =        0x43,
	DelAttr =        0x44,
	CallAttr =       0x45,
	Add =            0x46,
	Subtract =       0x47,
	Multiply =       0x48,
	Divide =         0x49,
	Modulo =         0x4a,
	Power =          0x4b,
	Equal =          0x4c,
	NotEqual =       0x4d,
	LessThan =       0x4e,
	GreaterThan =    0x4f,
	LessEqual =      0x50,
	GreaterEqual =   0x51,
	Compare =        0x52,
	CallAttrSimple = -0x41i8 as u8,
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
			"*" => Some(Self::Multiply),
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

	pub const fn arity_and_is_variable(self) -> (usize, bool) {
		(((self as u8 as i8) / 0x20).abs() as usize, (self as u8 as i8) < 0)
	}
}
