//! Types relating to Quest's virtual machine.
mod args;
pub mod block;
pub mod frame;
mod opcode;
mod source_location;

pub use args::Args;
pub use block::Block;
pub use frame::Frame;
pub use opcode::Opcode;
pub use source_location::SourceLocation;

/// The max amount of arguments a "simple function call" can have.
pub const MAX_ARGUMENTS_FOR_SIMPLE_CALL: usize = 16; // TODO: this needs to be verified everywhere, as ub is possible.

const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;
