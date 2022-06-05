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

const NUM_ARGUMENT_REGISTERS: usize = 16;
const COUNT_IS_NOT_ONE_BYTE_BUT_USIZE: u8 = i8::MAX as u8;
