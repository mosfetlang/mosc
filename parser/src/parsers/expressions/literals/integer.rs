use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::ParserNode;
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

/// A natural (without sign) number literal in the Mosfet language.
/// Can be written in binary(`0b`), octal(`0o`), decimal(`0d`) or hexadecimal(`0x`).
/// For decimal, the prefix can be omitted.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IntegerNumber {
    has_prefix: bool,
    radix: Radix,
    digits: Arc<Span>,
    span: Arc<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl IntegerNumber {
    // GETTERS ----------------------------------------------------------------

    /// Whether the number is prefixed or not.
    pub fn has_prefix(&self) -> bool {
        self.has_prefix
    }

    /// Whether the number is prefixed or not.
    pub fn prefix(&self) -> &'static str {
        if self.has_prefix {
            match self.radix {
                Radix::Binary => BINARY_PREFIX,
                Radix::Octal => OCTAL_PREFIX,
                Radix::Decimal => DECIMAL_PREFIX,
                Radix::Hexadecimal => HEXADECIMAL_PREFIX
            }
        } else {
            ""
        }
    }

    /// The radix in which the number is represented.
    pub fn radix(&self) -> &Radix {
        &self.radix
    }

    /// The number value.
    pub fn digits(&self) -> &Arc<Span> {
        &self.digits
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a prefixed `IntegerNumber` or a decimal without prefix.
    pub fn parse(reader: &mut Reader, context: &ParserContext) -> ParserResult<IntegerNumber> {
        cursor_manager(reader, |reader, init_cursor| {
            if reader.read(BINARY_PREFIX) {
                return Self::parse_binary(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = Arc::new(span);
                    number.has_prefix = true;
                    number
                });
            }

            if reader.read(OCTAL_PREFIX) {
                return Self::parse_octal(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = Arc::new(span);
                    number.has_prefix = true;
                    number
                });
            }

            if reader.read(HEXADECIMAL_PREFIX) {
                return Self::parse_hexadecimal(reader, context).map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = Arc::new(span);
                    number.has_prefix = true;
                    number
                });
            }

            // Decimal
            let has_prefix = reader.read(DECIMAL_PREFIX);

            Self::parse_decimal(reader, context).map(|mut number| {
                let span = reader.substring_to_current(&init_cursor);
                number.span = Arc::new(span);
                number.has_prefix = has_prefix;
                number
            })
        })
    }

    /// Parses a binary `IntegerNumber` without prefix.
    pub fn parse_binary(reader: &mut Reader, context: &ParserContext) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &BINARY_CHARS, Radix::Binary)
    }

    /// Parses an octal `IntegerNumber` without prefix.
    pub fn parse_octal(reader: &mut Reader, context: &ParserContext) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &OCTAL_CHARS, Radix::Octal)
    }

    /// Parses a decimal `IntegerNumber` without prefix.
    pub fn parse_decimal(reader: &mut Reader, context: &ParserContext) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &DECIMAL_CHARS, Radix::Decimal)
    }

    /// Parses an hexadecimal `IntegerNumber` without prefix.
    pub fn parse_hexadecimal(reader: &mut Reader, context: &ParserContext) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &HEXADECIMAL_CHARS, Radix::Hexadecimal)
    }

    /// Parses an `IntegerNumber` without prefix.
    fn parse_number(
        reader: &mut Reader,
        _context: &ParserContext,
        interval: &[RangeInclusive<char>],
        radix: Radix,
    ) -> ParserResult<IntegerNumber> {
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

            let digits = Arc::new(reader.substring_to_current(&init_cursor));
            Ok(IntegerNumber {
                has_prefix: false,
                radix,
                span: digits.clone(),
                digits,
            })
        })
    }

    // TODO check_start_with_separator_error
    // TODO check_end_with_separator_error
    // TODO check_leading_zeroes_warning
    // TODO check_leading_zeroes_warning
}

impl ParserNode for IntegerNumber {
    fn span(&self) -> &Arc<Span> {
        &self.span
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        // Decimal without prefix.
        let mut reader = Reader::from_str("25/rest");
        let number =
            IntegerNumber::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.content(), "25", "The content is incorrect");
        assert_eq!(number.digits().content(), "25", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");

        // Binary with prefix.
        let mut reader = Reader::from_str("0b10/rest");
        let number =
            IntegerNumber::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.content(), "0b10", "The content is incorrect");
        assert_eq!(number.digits().content(), "10", "The digits field is incorrect");
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");

        // Octal with prefix.
        let mut reader = Reader::from_str("0o74/rest");
        let number =
            IntegerNumber::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.content(), "0o74", "The content is incorrect");
        assert_eq!(number.digits().content(), "74", "The digits field is incorrect");
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");

        // Decimal with prefix.
        let mut reader = Reader::from_str("0d53/rest");
        let number =
            IntegerNumber::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.content(), "0d53", "The content is incorrect");
        assert_eq!(number.digits().content(), "53", "The digits field is incorrect");
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");

        // Hexadecimal with prefix.
        let mut reader = Reader::from_str("0x123/rest");
        let number =
            IntegerNumber::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.content(), "0x123", "The content is incorrect");
        assert_eq!(number.digits().content(), "123", "The digits field is incorrect");
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Hexadecimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_binary() {
        let mut reader = Reader::from_str("1010101010/rest");
        let number = IntegerNumber::parse_binary(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1010101010", "The content is incorrect");
        assert_eq!(number.digits().content(), "1010101010", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_binary_with_underscores() {
        let mut reader = Reader::from_str("101_01_____0101____0/rest");
        let number = IntegerNumber::parse_binary(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "101_01_____0101____0", "The content is incorrect");
        assert_eq!(number.digits().content(), "101_01_____0101____0", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_octal() {
        let mut reader = Reader::from_str("12345670/rest");
        let number = IntegerNumber::parse_octal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "12345670", "The content is incorrect");
        assert_eq!(number.digits().content(), "12345670", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_octal_with_underscores() {
        let mut reader = Reader::from_str("12_34_____56___70/rest");
        let number = IntegerNumber::parse_octal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "12_34_____56___70", "The content is incorrect");
        assert_eq!(number.digits().content(), "12_34_____56___70", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_decimal() {
        let mut reader = Reader::from_str("1234567890/rest");
        let number = IntegerNumber::parse_decimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1234567890", "The content is incorrect");
        assert_eq!(number.digits().content(), "1234567890", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_decimal_with_underscores() {
        let mut reader = Reader::from_str("1_234_____567___890/rest");
        let number = IntegerNumber::parse_decimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1_234_____567___890", "The content is incorrect");
        assert_eq!(number.digits().content(), "1_234_____567___890", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_hexadecimal() {
        let mut reader = Reader::from_str("1234567890abcdefABCDEF/rest");
        let number = IntegerNumber::parse_hexadecimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1234567890abcdefABCDEF", "The content is incorrect");
        assert_eq!(number.digits().content(), "1234567890abcdefABCDEF", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Hexadecimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_hexadecimal_with_underscores() {
        let mut reader = Reader::from_str("12_345678______90ab____cdefA____BCDEF/rest");
        let number = IntegerNumber::parse_hexadecimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.content(), "12_345678______90ab____cdefA____BCDEF", "The content is incorrect");
        assert_eq!(number.digits().content(), "12_345678______90ab____cdefA____BCDEF", "The digits field is incorrect");
        assert_eq!(number.has_prefix, false, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Hexadecimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("test");
        let error = IntegerNumber::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            error.variant_eq(&ParserError::NotFound),
            "The error is incorrect"
        );
        assert_eq!(reader.offset(), 0, "The offset is incorrect");

        // Check after prefix.
        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(prefix);
            let error = IntegerNumber::parse(&mut reader, &ParserContext::default())
                .expect_err("The parser must not succeed");

            assert!(
                error.variant_eq(&ParserError::NotFound),
                "The error is incorrect"
            );
            assert_eq!(reader.offset(), 0, "The offset is incorrect");
        }
    }
}
