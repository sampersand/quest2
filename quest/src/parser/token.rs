use super::{ErrorKind, Result, Span, Stream};
use std::fmt::{self, Debug, Formatter};

use crate::value::ty::{Float, Integer, Text};
use crate::value::Gc;

#[derive(Clone, Copy)]
pub struct Token<'a> {
	pub contents: TokenContents<'a>,
	pub span: Span<'a>,
}

impl Debug for Token<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Token")
				.field("contents", &self.contents)
				.field("span", &self.span)
				.finish()
		} else {
			Debug::fmt(&self.contents, f)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParenType {
	Round,
	Square,
	Curly,
}

#[derive(Debug, Clone, Copy)]
pub enum TokenContents<'a> {
	Text(Gc<Text>),
	Integer(Integer),
	Float(Float),

	LeftParen(ParenType),
	RightParen(ParenType),

	Period,
	Semicolon,
	Comma,
	ColonColon,

	Identifier(&'a str),
	Symbol(&'a str), // eg `+`, `**`, and user-definable ones too like `<$$>`.

	MacroIdentifier(&'a str),
	MacroLeftParen(ParenType),
}

fn strip_whitespace_and_comments(stream: &mut Stream<'_>) {
	while !stream.is_eof() {
		if stream.starts_with("\n__EOF__\n") {
			stream.set_eof();
			return;
		}

		if stream.take_if(char::is_whitespace).is_some() {
			continue;
		}

		// we only have single-line comments
		if stream.take_if(|c| '#' == c).is_some() {
			while let Some(chr) = stream.take() {
				if chr == '\n' {
					break;
				}
			}
		} else {
			break;
		}
	}
}

impl<'a> Token<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Option<Self>> {
		strip_whitespace_and_comments(stream);

		if stream.is_eof() {
			return Ok(None);
		}

		let start = stream.location();
		let contents = TokenContents::parse(stream)?;
		let end = stream.location();

		Ok(Some(Self {
			span: Span { start, end },
			contents,
		}))
	}
}

// TODO: maybe use unicode?
fn is_symbol_char(chr: char) -> bool {
	"~!@$%^&*-=+|\\;:,<.>/?".contains(chr)
}

fn take_identifier<'a>(stream: &mut Stream<'a>) -> &'a str {
	stream.take_while(|c| c.is_alphanumeric() || c == '_')
}

impl<'a> TokenContents<'a> {
	pub fn parse(stream: &mut Stream<'a>) -> Result<'a, Self> {
		match stream.peek().expect(".parse called with empty stream") {
			'(' => { stream.advance(); Ok(Self::LeftParen(ParenType::Round)) },
			'[' => { stream.advance(); Ok(Self::LeftParen(ParenType::Square)) },
			'{' => { stream.advance(); Ok(Self::LeftParen(ParenType::Curly)) },
			')' => { stream.advance(); Ok(Self::RightParen(ParenType::Round)) },
			']' => { stream.advance(); Ok(Self::RightParen(ParenType::Square)) },
			'}' => { stream.advance(); Ok(Self::RightParen(ParenType::Curly)) },
			'.' if !stream.peek2().map_or(false, is_symbol_char) => {
				stream.advance();
				Ok(Self::Period)
			},
			',' if !stream.peek2().map_or(false, is_symbol_char) => {
				stream.advance();
				Ok(Self::Period)
			},
			';' if !stream.peek2().map_or(false, is_symbol_char) => {
				stream.advance();
				Ok(Self::Semicolon)
			},
			':' if stream.peek2() == Some(':') && !stream.peek3().map_or(false, is_symbol_char) => {
				stream.advance();
				stream.advance();
				Ok(Self::ColonColon)
			},
			chr if chr.is_ascii_digit() => parse_number(stream),
			'-' | '+' if stream.peek2().map_or(false, |c| c.is_ascii_digit()) => parse_number(stream),
			'\'' | '"' => parse_text(stream),
			'$' => parse_macro(stream),
			chr if chr.is_alphabetic() => Ok(Self::Identifier(take_identifier(stream))),
			chr if is_symbol_char(chr) => Ok(Self::Symbol(stream.take_while(is_symbol_char))),
			other => Err(stream.error(ErrorKind::UnknownTokenStart(other))),
		}
	}
}

fn parse_macro<'a>(stream: &mut Stream<'a>) -> Result<'a, TokenContents<'a>> {
	let dollar = stream.take();
	debug_assert_eq!(dollar, Some('$'));

	match stream.peek() {
		Some('(') => Ok(TokenContents::MacroLeftParen(ParenType::Round)),
		Some('[') => Ok(TokenContents::MacroLeftParen(ParenType::Square)),
		Some('{') => Ok(TokenContents::MacroLeftParen(ParenType::Curly)),
		Some(c) if c.is_alphanumeric() => Ok(TokenContents::MacroIdentifier(take_identifier(stream))),
		_ => Ok(TokenContents::Symbol("$"))
	}
}

fn determine_base<'a>(stream: &mut Stream<'a>) -> Result<'a, u32> {
	if stream.take_if(|c| c == '0').is_none() {
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
fn parse_integer_base(stream: &mut Stream<'_>, base: u32, is_negative: bool) -> Integer {
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

	if is_negative {
		integer = -integer;
	}

	integer
}

fn parse_float<'a>(lhs: Integer, stream: &mut Stream<'a>) -> Result<'a, Float> {
	let mut float = lhs as Float;

	// OPTIMIZE: in the future, parsing a string should be handled by the rust stdlib or something.
	if stream.take_if(|c| c == '.').is_some() {
		let mut i = 0.1;

		while let Some(chr) = stream.take_if(|c| c.is_ascii_digit() || c == '_') {
			if chr == '_' {
				continue;
			}
			float += (chr.to_digit(10).unwrap() as Float) * i;
			i /= 10.0;
		}
	}

	let exponent = if stream.take_if(|c| c == 'e' || c == 'E').is_some() {
		let is_exp_neg = Some('-') == stream.take_if(|c| c == '-' || c=='+');
		parse_integer_base(stream, 10, is_exp_neg)
	} else {
		1
	};

	Ok(float * (10.0 as Float).powi(exponent as i32))
}

// Note that unary minus/plus are coalesced during constant joining.
fn parse_number<'a>(stream: &mut Stream<'a>) -> Result<'a, TokenContents<'a>> {
	let is_negative = Some('-') == stream.take_if(|c| c == '-' || c == '+');
	let base = determine_base(stream)?;
	let integer = parse_integer_base(stream, base, is_negative);

	let contents = match stream.peek() {
		Some('e' | 'E' | '.') if base == 10 => TokenContents::Float(parse_float(integer, stream)?),
		_ => TokenContents::Integer(integer),
	};

	match stream.peek() {
		Some(c) if c.is_alphanumeric() => {
			Err(stream.error(ErrorKind::BadCharacterAfterIntegerLiteral(c)))
		},
		_ => Ok(contents),
	}
}

fn next_hex<'a>(stream: &mut Stream<'a>) -> Result<'a, u32> {
	let chr = stream
		.take()
		.ok_or_else(|| stream.error(ErrorKind::UnterminatedQuote))?;

	chr.to_digit(16)
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

fn parse_text<'a>(stream: &mut Stream<'a>) -> Result<'a, TokenContents<'a>> {
	let quote = stream.take().unwrap();
	debug_assert!(quote == '\'' || quote == '"');

	// Nearly all string literals are going to be fairly small. So if we
	// preallocate an embedded `Text`, that'll cover nearly every case.
	let mut builder = Text::simple_builder();

	while let Some(chr) = stream.take() {
		// If it's the starting quote, then finish parsing.
		if chr == quote {
			return Ok(TokenContents::Text(builder.finish()));
		}

		// If it's not a backslash, then just insert the literal character in.
		if chr != '\\' {
			builder.push(chr);
			continue;
		}

		// It _is_ a backslash. What is it escaping?
		let escape = stream
			.take()
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
