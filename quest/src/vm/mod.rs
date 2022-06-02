mod args;
pub mod block;
pub mod bytecode;
mod frame;
mod source_location;

pub use args::Args;
pub use block::Block;
pub use bytecode::Opcode;
pub use frame::Frame;
pub use source_location::SourceLocation;

/// The max amount of arguments a "simple function call" can have.
pub const MAX_ARGUMENTS_FOR_SIMPLE_CALL: usize = 16;
const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;
