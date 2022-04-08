use super::{Stream, Token, Result, ErrorKind};
use crate::value::ty::Text;

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
				| next_hex(stream)?
			).ok_or_else(|| stream.error(ErrorKind::InvalidEscape))?,
		_ => return Err(stream.error(ErrorKind::InvalidEscape))
	})
}

pub fn parse_text<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<Token<'a>>> {
	let quote =
		if let Some(quote) = stream.take_if(|c| c == '\'' || c == '\"') {
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
