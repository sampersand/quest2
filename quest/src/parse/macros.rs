use crate::parse::{Parser, Result};

#[derive(Debug)]
pub struct Match {}

#[derive(Debug)]
pub struct Replacement {}

#[derive(Debug)]
pub struct Macro<'a> {
	group: Option<&'a str>,
	priority: Option<usize>,
	matches: Vec<Match>,
	replacements: Vec<Replacement>,
}

impl<'a> Macro<'a> {
	pub const fn new(
		group: Option<&'a str>,
		priority: Option<usize>,
		matches: Vec<Match>,
		replacements: Vec<Replacement>,
	) -> Self {
		Self {
			group,
			priority,
			matches,
			replacements,
		}
	}

	pub const fn group(&self) -> Option<&'a str> {
		self.group
	}

	pub const fn priority(&self) -> Option<usize> {
		self.priority
	}

	pub fn foo(&self) {
		let _ = self.replacements;
		let _ = self.matches;
	}
}


impl<'a> Macro<'a> {
	pub fn parse(parser: &mut Parser<'a>) -> Result<'a, Option<Self>> {
		if parser.take_if_contents(TokenContents::MacroIdentifier(1, "syntax"))?.is_none() {
			return Ok(None);
		}

		let group = parser.take_if(|c| c == '3');
		panic!();
		// while parser
		// 	.take_if_contents(TokenContents::RightParen(end))?
		// 	.is_none()

		// let args = BlockArgs::parse(parser)?;

		// if let Some(body) = Group::parse(parser, ParenType::Curly)? {
		// 	Ok(Some(Self { args, body }))
		// } else if args.is_some() {
		// 	panic!("todo: error because block args were given without a block");
		// } else {
		// 	Ok(None)
		// }
	}
}
