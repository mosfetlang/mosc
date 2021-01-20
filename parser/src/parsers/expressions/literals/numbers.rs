use std::ops::RangeInclusive;

use num_bigint::BigInt;
use num_rational::BigRational;

use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::utils::cursor_manager;
use crate::parsers::{ParserContext, ParserResult};

static BINARY_PREFIX: &str = "0b";
static OCTAL_PREFIX: &str = "0o";
static DECIMAL_PREFIX: &str = "0d";
static HEXADECIMAL_PREFIX: &str = "0x";
static BINARY_CHARS: [RangeInclusive<char>; 1] = ['0'..='1'];
static OCTAL_CHARS: [RangeInclusive<char>; 1] = ['0'..='7'];
static DECIMAL_CHARS: [RangeInclusive<char>; 1] = ['0'..='9'];
static HEXADECIMAL_CHARS: [RangeInclusive<char>; 3] = ['0'..='9', 'A'..='F', 'a'..='f'];
static SEPARATOR_RANGE: [RangeInclusive<char>; 1] = ['_'..='_'];

/// A number in the Mosfet language.
/// Can be written in binary(`0b`), octal(`0o`), decimal(`0d`) and hexadecimal(`0x`),
/// using their own prefix. For decimal can be omitted.
#[derive(Debug)]
pub struct Number {
    span: Span,
    number: BigRational,
}

impl Number {
    // GETTERS ----------------------------------------------------------------

    /// The span of the `Number`.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The number value.
    pub fn number(&self) -> &BigRational {
        &self.number
    }

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
                number: BigRational::from_integer(
                    BigInt::parse_bytes(span.content().replace("_", "").as_bytes(), radix).unwrap(),
                ),
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
    use num_bigint::ToBigInt;

    use super::*;

    #[test]
    fn test_parse_number() {
        // Decimal without prefix.
        let mut reader = Reader::from_str("25/rest");
        let number =
            Number::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.span().content(), "25", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&25).unwrap()),
            "The number is incorrect"
        );

        // Binary with prefix.
        let mut reader = Reader::from_str("0b10/rest");
        let number =
            Number::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.span().content(), "0b10", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0b10).unwrap()),
            "The number is incorrect"
        );

        // Octal with prefix.
        let mut reader = Reader::from_str("0o74/rest");
        let number =
            Number::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.span().content(), "0o74", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0o74).unwrap()),
            "The number is incorrect"
        );

        // Decimal with prefix.
        let mut reader = Reader::from_str("0d53/rest");
        let number =
            Number::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.span().content(), "0d53", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&53).unwrap()),
            "The number is incorrect"
        );

        // Hexadecimal with prefix.
        let mut reader = Reader::from_str("0x123/rest");
        let number =
            Number::parse(&mut reader, &ParserContext::default()).expect("The parser must succeed");

        assert_eq!(number.span().content(), "0x123", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0x123).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_binary() {
        let mut reader = Reader::from_str("10101010102/rest");
        let number = Number::parse_binary(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "1010101010",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0b1010101010).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_binary_with_underscores() {
        let mut reader = Reader::from_str("101_01_____0101____02/rest");
        let number = Number::parse_binary(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "101_01_____0101____0",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0b1010101010).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_octal() {
        let mut reader = Reader::from_str("12345670/rest");
        let number = Number::parse_octal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(number.span().content(), "12345670", "The span is incorrect");
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0o12345670).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_octal_with_underscores() {
        let mut reader = Reader::from_str("12_34_____56___70/rest");
        let number = Number::parse_octal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "12_34_____56___70",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&0o12345670).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_decimal() {
        let mut reader = Reader::from_str("1234567890/rest");
        let number = Number::parse_decimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "1234567890",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&1234567890).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_decimal_with_underscores() {
        let mut reader = Reader::from_str("1_234_____567___890/rest");
        let number = Number::parse_decimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "1_234_____567___890",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(ToBigInt::to_bigint(&1234567890).unwrap()),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_hexadecimal() {
        let mut reader = Reader::from_str("1234567890abcdefABCDEF/rest");
        let number = Number::parse_hexadecimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "1234567890abcdefABCDEF",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(
                ToBigInt::to_bigint(&(0x1234567890abcdefABCDEF as i128)).unwrap()
            ),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_hexadecimal_with_underscores() {
        let mut reader = Reader::from_str("12_345678______90ab____cdefA____BCDEF/rest");
        let number = Number::parse_hexadecimal(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            number.span().content(),
            "12_345678______90ab____cdefA____BCDEF",
            "The span is incorrect"
        );
        assert_eq!(
            number.number(),
            &BigRational::from_integer(
                ToBigInt::to_bigint(&(0x1234567890abcdefABCDEF as i128)).unwrap()
            ),
            "The number is incorrect"
        );
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("test");
        let error = Number::parse(&mut reader, &ParserContext::default())
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
            let error = Number::parse(&mut reader, &ParserContext::default())
                .expect_err("The parser must not succeed");

            assert!(
                error.variant_eq(&ParserError::NotFound),
                "The error is incorrect"
            );
            assert_eq!(reader.offset(), 0, "The offset is incorrect");
        }
    }
}
