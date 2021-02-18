use std::ops::RangeInclusive;

use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::{ParserContext, ParserResult};
use crate::parsers::utils::cursor_manager;

static BINARY_PREFIX: &str = "0b";
static OCTAL_PREFIX: &str = "0o";
static DECIMAL_PREFIX: &str = "0d";
static HEXADECIMAL_PREFIX: &str = "0x";
static BINARY_CHARS: &[RangeInclusive<char>] = &['0'..='1'];
static OCTAL_CHARS: &[RangeInclusive<char>] = &['0'..='7'];
static DECIMAL_CHARS: &[RangeInclusive<char>] = &['0'..='9'];
static HEXADECIMAL_CHARS: &[RangeInclusive<char>] = &['0'..='9', 'A'..='F', 'a'..='f'];
static SEPARATOR_RANGE: &[RangeInclusive<char>] = &['_'..='_'];

/// A number in the Mosfet language.
/// Can be written in binary(`0b`), octal(`0o`), decimal(`0d`) and hexadecimal(`0x`),
/// using their own prefix. For decimal can be omitted.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Number {
    span: Span,
    // number: BigRational,
}

impl Number {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        &self.span
    }

    // /// The number value.
    // pub fn number(&self) -> &BigRational {
    //     &self.number
    // }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a prefixed `Number` or a decimal without prefix.
    pub fn parse(reader: &mut Reader, context: &ParserContext) -> ParserResult<Number> {
        cursor_manager(reader, |reader, init_cursor| {
            if reader.read(BINARY_PREFIX) {
                return Self::parse_binary(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = span;
                    number
                });
            }

            if reader.read(OCTAL_PREFIX) {
                return Self::parse_octal(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = span;
                    number
                });
            }

            if reader.read(HEXADECIMAL_PREFIX) {
                return Self::parse_hexadecimal(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = span;
                    number
                });
            }

            // Decimal
            reader.read(DECIMAL_PREFIX);

            Self::parse_decimal(reader, context).map(|mut number| {
                let span = reader.substring_to_current(&init_cursor);
                number.span = span;
                number
            })
        })
    }

    /// Parses a binary `Number` without prefix.
    pub fn parse_binary(reader: &mut Reader, context: &ParserContext) -> ParserResult<Number> {
        Self::parse_number(reader, context, &BINARY_CHARS, 2)
    }

    /// Parses an octal `Number` without prefix.
    pub fn parse_octal(reader: &mut Reader, context: &ParserContext) -> ParserResult<Number> {
        Self::parse_number(reader, context, &OCTAL_CHARS, 8)
    }

    /// Parses a decimal `Number` without prefix.
    pub fn parse_decimal(reader: &mut Reader, context: &ParserContext) -> ParserResult<Number> {
        Self::parse_number(reader, context, &DECIMAL_CHARS, 10)
    }

    /// Parses an hexadecimal `Number` without prefix.
    pub fn parse_hexadecimal(reader: &mut Reader, context: &ParserContext) -> ParserResult<Number> {
        Self::parse_number(reader, context, &HEXADECIMAL_CHARS, 16)
    }

    /// Parses a `Number` without prefix.
    fn parse_number(
        reader: &mut Reader,
        _context: &ParserContext,
        interval: &[RangeInclusive<char>],
        radix: u32,
    ) -> ParserResult<Number> {
        cursor_manager(reader, |reader, init_cursor| {
            if let None = reader.read_many_of(interval) {
                return Err(ParserError::NotFound);
            }

            loop {
                let init_loop_cursor = reader.save();
                if let None = reader.read_many_of(&SEPARATOR_RANGE) {
                    break;
                }

                if let None = reader.read_many_of(interval) {
                    reader.restore(init_loop_cursor);
                    break;
                }
            }

            let span = reader.substring_to_current(&init_cursor);
            Ok(Number {
                // number: BigRational::from_integer(
                //     BigInt::parse_bytes(span.content().replace("_", "").as_bytes(), radix).unwrap(),
                // ),
                span,
            })
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    // use super::*;

// TODO
}
