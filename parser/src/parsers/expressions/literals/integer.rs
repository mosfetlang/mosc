use std::ops::RangeInclusive;
use std::sync::Arc;

use doclog::Color;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::utils::{
    cursor_manager, generate_error_log, generate_source_code, generate_warning_log,
};
use crate::parsers::{ParserResult, ParserResultError};
use crate::ParserNode;
use crate::{ParserError, ParserWarning};

pub static BINARY_PREFIX: &str = "0b";
pub static OCTAL_PREFIX: &str = "0o";
pub static DECIMAL_PREFIX: &str = "0d";
pub static HEXADECIMAL_PREFIX: &str = "0x";
pub static BINARY_DIGIT_CHARS: &[RangeInclusive<char>] = &['0'..='1'];
pub static OCTAL_DIGIT_CHARS: &[RangeInclusive<char>] = &['0'..='7'];
pub static DECIMAL_DIGIT_CHARS: &[RangeInclusive<char>] = &['0'..='9'];
pub static HEXADECIMAL_DIGIT_CHARS: &[RangeInclusive<char>] = &['0'..='9', 'A'..='F', 'a'..='f'];
pub static SEPARATOR_RANGE: &[RangeInclusive<char>] = &['_'..='_'];

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Radix {
    // GETTERS ----------------------------------------------------------------

    pub fn prefix_str(&self) -> &'static str {
        match self {
            Radix::Binary => BINARY_PREFIX,
            Radix::Octal => OCTAL_PREFIX,
            Radix::Decimal => DECIMAL_PREFIX,
            Radix::Hexadecimal => HEXADECIMAL_PREFIX,
        }
    }

    pub fn digit_chars(&self) -> &[RangeInclusive<char>] {
        match self {
            Radix::Binary => BINARY_DIGIT_CHARS,
            Radix::Octal => OCTAL_DIGIT_CHARS,
            Radix::Decimal => DECIMAL_DIGIT_CHARS,
            Radix::Hexadecimal => HEXADECIMAL_DIGIT_CHARS,
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

/// A natural (without sign) number literal in the Mosfet language.
/// Can be written in binary(`0b`), octal(`0o`), decimal(`0d`) or hexadecimal(`0x`).
/// For decimal, the prefix can be omitted.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IntegerNumber {
    span: Arc<Span>,
    has_prefix: bool,
    radix: Radix,
    digits: Arc<Span>,
}

impl IntegerNumber {
    // GETTERS ----------------------------------------------------------------

    /// Whether the number is prefixed or not.
    pub fn has_prefix(&self) -> bool {
        self.has_prefix
    }

    /// The prefix of the number as str.
    pub fn prefix_str(&self) -> &'static str {
        if self.has_prefix {
            self.radix.prefix_str()
        } else {
            ""
        }
    }

    /// The radix in which the number is represented.
    pub fn radix(&self) -> &Radix {
        &self.radix
    }

    /// The digits of the number.
    pub fn digits(&self) -> &Arc<Span> {
        &self.digits
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a prefixed `IntegerNumber` or a decimal without prefix.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<IntegerNumber> {
        cursor_manager(reader, |reader, init_cursor| {
            if reader.read(BINARY_PREFIX) {
                return Self::parse_number(
                    reader,
                    context,
                    &BINARY_DIGIT_CHARS,
                    Radix::Binary,
                    true,
                )
                .map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = Arc::new(span);
                    number.has_prefix = true;
                    number
                });
            }

            if reader.read(OCTAL_PREFIX) {
                return Self::parse_number(reader, context, &OCTAL_DIGIT_CHARS, Radix::Octal, true)
                    .map(|mut number| {
                        let span = reader.substring_to_current(&init_cursor);
                        number.span = Arc::new(span);
                        number.has_prefix = true;
                        number
                    });
            }

            if reader.read(HEXADECIMAL_PREFIX) {
                return Self::parse_number(
                    reader,
                    context,
                    &HEXADECIMAL_DIGIT_CHARS,
                    Radix::Hexadecimal,
                    true,
                )
                .map(|mut number| {
                    let span = reader.substring_to_current(&init_cursor);
                    number.span = Arc::new(span);
                    number.has_prefix = true;
                    number
                });
            }

            // Decimal
            let has_prefix = reader.read(DECIMAL_PREFIX);

            Self::parse_number(
                reader,
                context,
                &DECIMAL_DIGIT_CHARS,
                Radix::Decimal,
                has_prefix,
            )
            .map(|mut number| {
                let span = reader.substring_to_current(&init_cursor);
                number.span = Arc::new(span);
                number.has_prefix = has_prefix;
                number
            })
        })
    }

    /// Parses a binary `IntegerNumber` without prefix.
    pub fn parse_binary(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &BINARY_DIGIT_CHARS, Radix::Binary, false)
    }

    /// Parses an octal `IntegerNumber` without prefix.
    pub fn parse_octal(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &OCTAL_DIGIT_CHARS, Radix::Octal, false)
    }

    /// Parses a decimal `IntegerNumber` without prefix.
    pub fn parse_decimal(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<IntegerNumber> {
        Self::parse_number(reader, context, &DECIMAL_DIGIT_CHARS, Radix::Decimal, false)
    }

    /// Parses an hexadecimal `IntegerNumber` without prefix.
    pub fn parse_hexadecimal(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<IntegerNumber> {
        Self::parse_number(
            reader,
            context,
            &HEXADECIMAL_DIGIT_CHARS,
            Radix::Hexadecimal,
            false,
        )
    }

    /// Parses an `IntegerNumber` without prefix.
    fn parse_number(
        reader: &mut Reader,
        context: &mut ParserContext,
        digit_interval: &[RangeInclusive<char>],
        radix: Radix,
        has_prefix: bool,
    ) -> ParserResult<IntegerNumber> {
        cursor_manager(reader, |reader, init_cursor| {
            if let None = reader.read_many_of(digit_interval) {
                if has_prefix {
                    // Error: separator after prefix.
                    let prefix = radix.prefix_str();

                    if reader.read_one_of(&SEPARATOR_RANGE).is_some() {
                        context.add_message(generate_error_log(
                            ParserError::NumberWithSeparatorAfterPrefix,
                            format!(
                                "A number cannot start with a separator '{}' after the prefix '{}'",
                                SEPARATOR_RANGE.first().unwrap().start(),
                                prefix
                            ),
                            |log| {
                                generate_source_code(log, &reader, |doc| {
                                    doc.highlight_section(
                                        (init_cursor.offset() - prefix.len())..init_cursor.offset(),
                                        None,
                                        Some(Color::Magenta),
                                    )
                                    .highlight_section_str(
                                        init_cursor.offset()..reader.offset(),
                                        Some("Remove this token"),
                                        None,
                                    )
                                })
                            },
                        ));

                        return Err(ParserResultError::Error);
                    }

                    // Error: missing digits after prefix.
                    context.add_message(generate_error_log(
                        ParserError::NumberWithoutDigitsAfterPrefix,
                        format!(
                            "At least one digit was expected after the prefix '{}'",
                            prefix
                        ),
                        |log| {
                            generate_source_code(log, &reader, |doc| {
                                doc.highlight_section(
                                    (init_cursor.offset() - prefix.len())..init_cursor.offset(),
                                    None,
                                    Some(Color::Magenta),
                                )
                                .highlight_cursor_str(
                                    reader.offset(),
                                    Some("Add a digit here, e.g. 0"),
                                    None,
                                )
                            })
                        },
                    ));

                    return Err(ParserResultError::Error);
                }

                return Err(ParserResultError::NotFound);
            }

            loop {
                let init_loop_cursor = reader.save_cursor();
                if let None = reader.read_many_of(&SEPARATOR_RANGE) {
                    break;
                }

                if let None = reader.read_many_of(digit_interval) {
                    reader.restore(init_loop_cursor);
                    break;
                }
            }

            let digits = Arc::new(reader.substring_to_current(&init_cursor));
            let result = IntegerNumber {
                has_prefix,
                radix,
                span: digits.clone(),
                digits,
            };

            Self::check_leading_zeroes(reader, context, &result.digits, result.prefix_str());

            Ok(result)
        })
    }

    fn check_leading_zeroes(
        reader: &mut Reader,
        context: &mut ParserContext,
        digits: &Arc<Span>,
        prefix: &str,
    ) {
        if context.ignore().number_leading_zeroes {
            return;
        }

        let content = digits.content();
        let mut new_content = content.trim_start_matches("0");

        if new_content.len() == content.len() {
            return;
        }

        let mut number_of_zeroes = content.len() - new_content.len();

        if new_content.len() == 0 {
            if number_of_zeroes == 1 {
                // Ignore because number is equal to 0
                return;
            } else {
                new_content = "0";
                number_of_zeroes -= 1;
            }
        };

        context.add_message(generate_warning_log(
            ParserWarning::NumberWithLeadingZeroes,
            "Leading zeroes are unnecessary".to_string(),
            |log| {
                generate_source_code(log, &reader, |doc| {
                    let doc = if prefix.len() != 0 {
                        doc.highlight_section(
                            (digits.start_cursor().offset() - prefix.len())
                                ..digits.start_cursor().offset(),
                            None,
                            Some(Color::Magenta),
                        )
                    } else {
                        doc
                    };

                    doc.highlight_section_str(
                        digits.start_cursor().offset()
                            ..(digits.start_cursor().offset() + number_of_zeroes),
                        Some(if number_of_zeroes == 1 {
                            "Remove this zero"
                        } else {
                            "Remove these zeroes"
                        }),
                        None,
                    )
                    .highlight_section(
                        (digits.end_cursor().offset() - new_content.len())
                            ..digits.end_cursor().offset(),
                        None,
                        Some(Color::Magenta),
                    )
                })
            },
        ));
    }
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
    use crate::test::{assert_error, assert_warning};
    use crate::ParserIgnoreConfig;

    use super::*;

    #[test]
    fn test_parse() {
        // Decimal without prefix.
        let mut reader = Reader::from_str("25/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "25", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "25",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");

        // Binary with prefix.
        let mut reader = Reader::from_str("0b10/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "0b10", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "10",
            "The digits field is incorrect"
        );
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");

        // Octal with prefix.
        let mut reader = Reader::from_str("0o74/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "0o74", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "74",
            "The digits field is incorrect"
        );
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");

        // Decimal with prefix.
        let mut reader = Reader::from_str("0d53/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "0d53", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "53",
            "The digits field is incorrect"
        );
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");

        // Hexadecimal with prefix.
        let mut reader = Reader::from_str("0x123/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "0x123", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "123",
            "The digits field is incorrect"
        );
        assert_eq!(number.has_prefix, true, "The has_prefix field is incorrect");
        assert_eq!(
            number.radix,
            Radix::Hexadecimal,
            "The radix field is incorrect"
        );
    }

    #[test]
    fn test_parse_binary() {
        let mut reader = Reader::from_str("1010101010/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_binary(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1010101010", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "1010101010",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_binary_with_underscores() {
        let mut reader = Reader::from_str("101_01_____0101____0/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_binary(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            number.content(),
            "101_01_____0101____0",
            "The content is incorrect"
        );
        assert_eq!(
            number.digits().content(),
            "101_01_____0101____0",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Binary, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_octal() {
        let mut reader = Reader::from_str("12345670/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse_octal(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(number.content(), "12345670", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "12345670",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_octal_with_underscores() {
        let mut reader = Reader::from_str("12_34_____56___70/rest");
        let mut context = ParserContext::default();
        let number =
            IntegerNumber::parse_octal(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            number.content(),
            "12_34_____56___70",
            "The content is incorrect"
        );
        assert_eq!(
            number.digits().content(),
            "12_34_____56___70",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Octal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_decimal() {
        let mut reader = Reader::from_str("1234567890/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_decimal(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(number.content(), "1234567890", "The content is incorrect");
        assert_eq!(
            number.digits().content(),
            "1234567890",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_decimal_with_underscores() {
        let mut reader = Reader::from_str("1_234_____567___890/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_decimal(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            number.content(),
            "1_234_____567___890",
            "The content is incorrect"
        );
        assert_eq!(
            number.digits().content(),
            "1_234_____567___890",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(number.radix, Radix::Decimal, "The radix field is incorrect");
    }

    #[test]
    fn test_parse_hexadecimal() {
        let mut reader = Reader::from_str("1234567890abcdefABCDEF/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_hexadecimal(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            number.content(),
            "1234567890abcdefABCDEF",
            "The content is incorrect"
        );
        assert_eq!(
            number.digits().content(),
            "1234567890abcdefABCDEF",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(
            number.radix,
            Radix::Hexadecimal,
            "The radix field is incorrect"
        );
    }

    #[test]
    fn test_parse_hexadecimal_with_underscores() {
        let mut reader = Reader::from_str("12_345678______90ab____cdefA____BCDEF/rest");
        let mut context = ParserContext::default();
        let number = IntegerNumber::parse_hexadecimal(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            number.content(),
            "12_345678______90ab____cdefA____BCDEF",
            "The content is incorrect"
        );
        assert_eq!(
            number.digits().content(),
            "12_345678______90ab____cdefA____BCDEF",
            "The digits field is incorrect"
        );
        assert_eq!(
            number.has_prefix, false,
            "The has_prefix field is incorrect"
        );
        assert_eq!(
            number.radix,
            Radix::Hexadecimal,
            "The radix field is incorrect"
        );
    }

    #[test]
    fn test_number_with_separator_after_prefix() {
        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(
                format!("{}{}", prefix, SEPARATOR_RANGE.last().unwrap().start()).as_str(),
            );
            let mut context = ParserContext::default();
            let error = IntegerNumber::parse(&mut reader, &mut context)
                .expect_err("The parser must not succeed");

            assert_error(
                &context,
                &error,
                ParserError::NumberWithSeparatorAfterPrefix,
            );
        }
    }

    #[test]
    fn test_number_without_digits_after_prefix() {
        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(prefix);
            let mut context = ParserContext::default();
            let error = IntegerNumber::parse(&mut reader, &mut context)
                .expect_err("The parser must not succeed");

            assert_error(
                &context,
                &error,
                ParserError::NumberWithoutDigitsAfterPrefix,
            );
        }
    }

    #[test]
    fn test_warning_leading_zeroes() {
        let mut reader = Reader::from_str("000");
        let mut context = ParserContext::default();
        IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_warning(&context, ParserWarning::NumberWithLeadingZeroes);

        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(format!("{}00", prefix).as_str());
            let mut context = ParserContext::default();
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

            assert_warning(&context, ParserWarning::NumberWithLeadingZeroes);
        }
    }

    #[test]
    fn test_ignore_warning_leading_zeroes() {
        let mut reader = Reader::from_str("000");
        let mut ignore = ParserIgnoreConfig::new();
        ignore.number_leading_zeroes = true;

        let mut context = ParserContext::new(ignore);
        IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(context.messages().len(), 0, "There must no be messages");
    }

    #[test]
    fn test_warning_leading_zeroes_ignores_ok_numbers() {
        for number in &["0", "1", "10101"] {
            let mut reader = Reader::from_str(number);
            let mut context = ParserContext::default();
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

            assert_eq!(context.messages().len(), 0, "There must no be messages");

            for prefix in &[
                BINARY_PREFIX,
                OCTAL_PREFIX,
                DECIMAL_PREFIX,
                HEXADECIMAL_PREFIX,
            ] {
                let mut reader = Reader::from_str(format!("{}{}", prefix, number).as_str());
                let mut context = ParserContext::default();
                IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

                assert_eq!(context.messages().len(), 0, "There must no be messages");
            }
        }
    }
}
