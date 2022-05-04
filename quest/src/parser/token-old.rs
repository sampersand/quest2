use super::{ErrorKind, Result, Span, Stream};
use crate::value::ty::{Float, Integer, Text};
use crate::value::Gc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParenType {
	Round,
	Curly,
	Square,
}

#[derive(Debug, Clone, Copy)]
pub enum Token<'a> {
	Text(Gc<Text>),
	Integer(Integer),
	Float(Float),
	Symbol(&'a str), // eg `+`, `**`, etc. user-definable.
	Identifier(&'a str),
	LeftParen(ParenType),
	RightParen(ParenType),

	MacroIdentifier(&'a str),
	MacroLeftParen(ParenType),
}

#[derive(Debug)]
pub struct SpannedToken<'a> {
	pub token: Token<'a>,
	pub span: Span<'a>,
}

fn strip_whitespace_and_comments(stream: &mut Stream<'_>) {
	loop {
		stream.take_while(char::is_whitespace);

		if stream.take_str("__EOF__") {
			stream.take_while(|_| true);
			break;
		}

		if !stream.take_if_chr('#') {
			return;
		}

		stream.take_while(|c| c != '\n');
	}
}

// TODO: maybe use unicode?
fn is_able_to_compose_an_operator(c: char) -> bool {
	"~!@$%^&*-=+|\\;:,<.>/?".contains(c)
}

fn next_hex<'a>(stream: &mut Stream<'a>) -> Result<'a, u32> {
	stream
		.next_char()
		.ok_or_else(|| stream.error(ErrorKind::UnterminatedQuote))?
		.to_digit(16)
		.ok_or_else(|| stream.error(ErrorKind::InvalidEscape))
}

fn double_quote_escape<'a>(escape: char, stream: &mut Stream<'a>) -> Result<'a, char> {
	Ok(match escape {
		'\'' | '\"' | '\\' => escape,
		'n' => '\n',
		't' => '\t',
		'r' => '\r',
		'f' => '\x0c',
		'x' => char::from_u32((next_hex(stream)? << 4) | next_hex(stream)?)
			.ok_or_else(|| stream.error(ErrorKind::InvalidEscape))?,
		'u' => char::from_u32(
			(next_hex(stream)? << 12)
				| (next_hex(stream)? << 8)
				| (next_hex(stream)? << 4)
				| next_hex(stream)?,
		)
		.ok_or_else(|| stream.error(ErrorKind::InvalidEscape))?,
		_ => return Err(stream.error(ErrorKind::InvalidEscape)),
	})
}

fn parse_text<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<Token<'a>>> {
	let quote = if let Some(quote) = stream.take_if(|c| c == '\'' || c == '\"') {
		quote
	} else {
		return Ok(None);
	};

	// Nearly all string literals are going to be fairly small. So if we
	// preallocate an embedded `Text`, that'll cover nearly every case.
	let mut builder = Text::simple_builder();

	while let Some(chr) = stream.next_char() {
		// If it's the starting quote, then finish parsing.
		if chr == quote {
			return Ok(Some(Token::Text(builder.finish())));
		}

		// If it's not a backslash, then just insert the literal character in.
		if chr != '\\' {
			builder.push(chr);
			continue;
		}

		// It _is_ a backslash. What is it escaping?
		let escape = stream
			.next_char()
			.ok_or_else(|| stream.error(ErrorKind::UnterminatedQuote))?;

		if quote == '\'' {
			// If we're single quoted, only `'`, `"`, and `\` are recognized as escapes.
			// Everything else is taken as a literal `\` followed by the character.
			if !"\'\"\\".contains(escape) {
				builder.push('\\');
			}

			builder.push(escape);
			continue;
		}

		// Now we're double quoted, so actually perform all those escapes
		debug_assert_eq!(quote, '\"');
		builder.push(double_quote_escape(escape, stream)?);
	}

	// If we reach down here, it means we hit EOF before the end quote was encountered.
	Err(stream.error(ErrorKind::UnterminatedQuote))
}


// Note that unary minus/plus are coalesced during constant joining.
fn parse_integer<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<Token<'a>>> {
	if !stream.peek().map_or(false, |c| c.is_ascii_digit()) {
		return Ok(None);
	}

	let base = determine_base(stream)?;
	let integer = parse_integer_base(stream, base);

	if stream.peek().map_or(false, |c| c.is_alphanumeric()) {
		Err(stream.error(ErrorKind::BadCharacter))
	} else {
		Ok(Some(Token::Integer(integer)))
	}
}

fn determine_base<'a>(stream: &mut Stream<'a>) -> Result<'a, u32> {
	if !stream.take_if_chr('0') {
		return Ok(10);
	}

	match stream.take_if(|c| "xXoObBdDuU".contains(c)) {
		Some('x' | 'X') => Ok(16),
		Some('o' | 'O') => Ok(8),
		Some('b' | 'B') => Ok(2),
		Some('u' | 'U') => todo!("custom base"),
		Some('d' | 'D') => Ok(10),
		Some(_) => unreachable!(),
		None => Err(stream.error(ErrorKind::UnexpectedEOF)),
	}
}

// This is _almost_ like parsing a string in a specific base, except we allow `_`s within numbers,
// which are stripped.
fn parse_integer_base(stream: &mut Stream<'_>, base: u32) -> Integer {
	let mut integer = 0;

	while let Some(chr) = stream.peek() {
		if let Some(digit) = chr.to_digit(base) {
			integer *= base as Integer;
			integer += digit as Integer;
		} else if chr != '_' {
			break;
		}

		stream.advance();
	}

	integer
}


impl<'a> Token<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Self> {
		match stream.peek().expect("we already checked for eof") {
			c if c.is_whitespace() || c == '#' => {
				unreachable!("we already stripped whitespace & comments")
			},
			'(' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Round))
			},
			')' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Round))
			},
			'[' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Square))
			},
			']' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Square))
			},
			'{' => {
				stream.advance();
				Ok(Self::LeftParen(ParenType::Curly))
			},
			'}' => {
				stream.advance();
				Ok(Self::RightParen(ParenType::Curly))
			},
			'\'' | '\"' => parse_text(stream).map(Option::unwrap),
			'0'..='9' => parse_integer(stream).map(Option::unwrap), // technically should parse floats too...
			'$' => {
				stream.advance();
				if stream.take_if_chr('[') {
					return Ok(Self::MacroLeftParen(ParenType::Square));
				}

				let ident = stream.take_while(|c| c.is_alphanumeric() || c == '_');
				if ident.is_empty() {
					Ok(Self::Symbol("$"))
				} else {
					Ok(Self::MacroIdentifier(ident))
				}
			}
			a if a.is_alphabetic() || a == '_' => {
				Ok(Self::Identifier(stream.take_while(|c| c.is_alphanumeric() || c == '_')))
			},
			a if is_able_to_compose_an_operator(a) => {
				Ok(Self::Symbol(stream.take_while(is_able_to_compose_an_operator)))
			},
			other => panic!("todo: return an error for unknown kind {:?}", other),
		}
	}
}

impl<'a> SpannedToken<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Option<Self>> {
		strip_whitespace_and_comments(stream);

		if stream.is_eof() {
			return Ok(None);
		}

		let start = stream.span_start();
		let token = Token::parse(stream)?;

		Ok(Some(Self {
			token,
			span: start.finish(stream),
		}))
	}
}
