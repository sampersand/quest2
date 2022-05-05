mod block_literal;
mod expression;

pub use block_literal::BlockLiteral;
pub use expression::Expression;

pub trait Plugin<'a>: Sized {
	fn parse(parser: &mut super::Parser<'a>) -> super::Result<'a, Option<Self>>;
	fn compile(self, builder: &mut crate::vm::block::Builder);
}

// pub trait Pattern<'a> {
// 	type Output: Compilable;

// 	fn try_match(parser: &mut super::Parser<'a>) -> super::Result<'a, Option<
// }
