use super::{ErrorKind, Result, Stream, Token};
use crate::value::ty::Integer;

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

// Note that unary minus/plus are coalesced during constant joining.
pub fn parse_integer<'a>(stream: &mut Stream<'a>) -> Result<'a, Option<Token<'a>>> {
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
