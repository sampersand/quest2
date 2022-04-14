#![allow(unused)]

use crate::vm::SourceLocation;
use crate::AnyValue;

type Local = usize;
type Offset = isize;

#[derive(Debug)]
pub enum Bytecode {

}

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
	Return,

	ConstLoad,
	GetAttr,
	HasAttr,
	SetAttr,
	DelAttr,
	CallAttr,

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

// pub struct Bytecode<'a>(&'a [u8]);
// pub struct BytecodeIter<'a>(Bytecode<'a>);



// // impl Bytecode

// // pub union ByteCode {
// // 	small_local: u8,
// // 	small_offset: i8,
// // // 	enum sq_opcode opcode;
// // // 	unsigned index;
// // // 	enum sq_interrupt interrupt;
// // // 	unsigned count;

// // }

// #[derive(Debug)]
// pub enum ByteCode {
// 	NoOp,

// 	Mov { src: Local, dst: Local },
// 	Jmp { offset: Offset },
// 	JmpFalse { cond: Local, offset: Offset },
// 	JmpTrue { cond: Local, offset: Offset },
// 	Call { func: Local, dst: Local, args: Box<[Local]> },
// 	Return { value: Local, from: Local },

// 	ConstantLoad { index: usize },
// 	GetAttr { obj: Local, attr: Local, dst: Local },
// 	HasAttr { obj: Local, attr: Local, dst: Local },
// 	SetAttr { obj: Local, attr: Local, value: Local },
// 	DelAttr { obj: Local, attr: Local, dst: Local },
// 	CallAttr { obj: Local, attr: Local, dst: Local, args: Box<[Local]> },

// 	Not { rhs: Local, dst: Local },
// 	Negate { rhs: Local, dst: Local },
// 	Equal { lhs: Local, rhs: Local, dst: Local },
// 	NotEqual { lhs: Local, rhs: Local, dst: Local },
// 	LessThan { lhs: Local, rhs: Local, dst: Local },
// 	GreaterThan { lhs: Local, rhs: Local, dst: Local },
// 	LessEqual { lhs: Local, rhs: Local, dst: Local },
// 	GreaterEqual { lhs: Local, rhs: Local, dst: Local },
// 	Compare { lhs: Local, rhs: Local, dst: Local },
// 	Add { lhs: Local, rhs: Local, dst: Local },
// 	Subtract { lhs: Local, rhs: Local, dst: Local },
// 	Multuply { lhs: Local, rhs: Local, dst: Local },
// 	Divide { lhs: Local, rhs: Local, dst: Local },
// 	Modulo { lhs: Local, rhs: Local, dst: Local },
// 	Power { lhs: Local, rhs: Local, dst: Local },
// 	Index { obj: Local, key: Local, dst: Local },
// 	IndexAssign { obj: Local, key: Local, value: Local },
// }

// // union sq_bytecode {
// // 	enum sq_opcode opcode;
// // 	unsigned index;
// // 	enum sq_interrupt interrupt;
// // 	unsigned count;
// // };

// // const char *sq_interrupt_repr(enum sq_interrupt interrupt);
// // const char *sq_opcode_repr(enum sq_opcode opcode);

// // #endif /* !SQ_BYTECODE_H */



// /*enum sq_interrupt {
// 	SQ_INT_UNDEFINED    = 0x00,
// 	SQ_INT_TONUMERAL    = 0x01, // [A,DST] DST <- A.to_numeral()
// 	SQ_INT_TOTEXT       = 0x02, // [A,DST] DST <- A.to_text()
// 	SQ_INT_TOVERACITY   = 0x03, // [A,DST] DST <- A.to_veracity()
// 	SQ_INT_TOBOOK       = 0x04, // [A,DST] DST <- A.to_book()
// 	SQ_INT_TOCODEX      = 0x05, // [A,DST] DST <- A.to_codex()
// 	SQ_INT_KINDOF       = 0x06, // [A,DST] DST <- A.genus

// 	SQ_INT_PRINT        = 0x10, // [A,DST] Print `A`, DST <- ni
// 	SQ_INT_PRINTLN      = 0x11, // [A,DST] Print `A` with a newline, DST <- ni
// 	SQ_INT_DUMP         = 0x12, // [A,DST] Dumps out `A`, DST <- A
// 	SQ_INT_PROMPT       = 0x13, // [DST] DST <- next line from stdin
// 	SQ_INT_SYSTEM       = 0x14, // [CMD,DST] DST <- stdout of running `cmd`.
// 	SQ_INT_EXIT         = 0x15, // [CODE] Exits with the given code.
// 	SQ_INT_RANDOM       = 0x16, // [DST] DST <- random numeral

// 	SQ_INT_SUBSTR       = 0x20, // [A,B,C,DST] DST <- A[B..B+C]
// 	SQ_INT_LENGTH       = 0x21, // [A,DST] DST <- length A: book/codex/text

// 	SQ_INT_CODEX_NEW    = 0x30, // [N,...,DST] DST <- N key-value pairs.
// 	SQ_INT_BOOK_NEW     = 0x31, // [N,...,DST] DST <- N-length array.
// 	SQ_INT_ARRAY_INSERT = 0x32, // [A,B,C,DST] A.insert(len=B,pos=C); (Stores in DST, though this is not intended)
// 	SQ_INT_ARRAY_DELETE = 0x33, // [A,B,DST] DST <- A.delete(B)

// 	SQ_INT_ARABIC       = 0x40, // [A,DST] DST <- A.to_numeral().arabic()
// 	SQ_INT_ROMAN        = 0x41, // [A,DST] DST <- A.to_numeral().roman()

// 	// temporary hacks until we get kingdoms working.
// 	SQ_INT_FOPEN,
// 	SQ_INT_FCLOSE,
// 	SQ_INT_FREAD,
// 	SQ_INT_FREADALL,
// 	SQ_INT_FWRITE,
// 	SQ_INT_FTELL,
// 	SQ_INT_FSEEK,

// 	// ASCII
// 	SQ_INT_ASCII,
// };
// */
