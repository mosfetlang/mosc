use std::sync::Arc;

use doclog::Color;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::expressions::literals::integer::{IntegerNumber, Radix, SEPARATOR_RANGE};
use crate::parsers::utils::{cursor_manager, generate_source_code, generate_warning_log};
use crate::parsers::ParserResult;
use crate::{ParserNode, ParserWarning};

static DECIMAL_SEPARATOR: &str = ".";

/// A number in the Mosfet language.
/// Can be written in binary(`0b`), octal(`0o`), decimal(`0d`) and hexadecimal(`0x`),
/// using their own prefix. For decimal can be omitted.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Number {
    span: Arc<Span>,
    integer: IntegerNumber,
    decimal_digits: Option<Arc<Span>>,
}

impl Number {
    // GETTERS ----------------------------------------------------------------

    /// Whether the number is prefixed or not.
    pub fn has_prefix(&self) -> bool {
        self.integer.has_prefix()
    }

    /// The prefix of the number as str.
    pub fn prefix_str(&self) -> &'static str {
        self.integer.prefix_str()
    }

    /// The radix in which the number is represented.
    pub fn radix(&self) -> &Radix {
        &self.integer.radix()
    }

    /// The digits belonging to the integer part of the number.
    pub fn integer_digits(&self) -> &Arc<Span> {
        &self.integer.digits()
    }

    /// The digits belonging to the decimal part of the number.
    pub fn decimal_digits(&self) -> &Option<Arc<Span>> {
        &self.decimal_digits
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a prefixed `Number` or a decimal without prefix.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<Number> {
        cursor_manager(reader, |reader, init_cursor| {
            let integer_part = IntegerNumber::parse(reader, context)?;

            let pre_decimal_cursor = reader.save_cursor();
            if !reader.read(DECIMAL_SEPARATOR) {
                return Ok(Number {
                    integer: integer_part,
                    decimal_digits: None,
                    span: Arc::new(reader.substring_to_current(init_cursor)),
                });
            }

            let post_decimal_cursor = reader.save_cursor();
            let digit_interval = integer_part.radix().digit_chars();
            if let None = reader.read_many_of(digit_interval) {
                reader.restore(pre_decimal_cursor);
                return Ok(Number {
                    integer: integer_part,
                    decimal_digits: None,
                    span: Arc::new(reader.substring_to_current(init_cursor)),
                });
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

            let result = Number {
                integer: integer_part,
                decimal_digits: Some(Arc::new(reader.substring_to_current(&post_decimal_cursor))),
                span: Arc::new(reader.substring_to_current(init_cursor)),
            };

            Self::check_trailing_zeroes(reader, context, &result);

            Ok(result)
        })
    }

    fn check_trailing_zeroes(reader: &mut Reader, context: &mut ParserContext, number: &Number) {
        if context.ignore().number_trailing_zeroes {
            return;
        }

        let decimal_digits = number.decimal_digits.as_ref().unwrap();
        let content = decimal_digits.content();
        let new_content = content.trim_end_matches("0");
        let mut number_of_zeroes = content.len() - new_content.len();

        if new_content.len() == 0 {
            if number_of_zeroes == 1 {
                // Ignore because number is equal to X.0
                return;
            } else {
                number_of_zeroes -= 1;
            }
        };

        context.add_message(generate_warning_log(
            ParserWarning::NumberWithTrailingZeroes,
            "Trailing zeroes are unnecessary".to_string(),
            |log| {
                generate_source_code(log, &reader, |doc| {
                    doc.highlight_section(
                        number.span.start_cursor().offset()
                            ..(decimal_digits.end_cursor().offset() - number_of_zeroes),
                        None,
                        Some(Color::Magenta),
                    )
                    .highlight_section_str(
                        (decimal_digits.end_cursor().offset() - number_of_zeroes)
                            ..decimal_digits.end_cursor().offset(),
                        Some(if number_of_zeroes == 1 {
                            "Remove this zero"
                        } else {
                            "Remove these zeroes"
                        }),
                        None,
                    )
                })
            },
        ));
    }
}

impl ParserNode for Number {
    fn span(&self) -> &Arc<Span> {
        &self.span
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::parsers::expressions::literals::integer::{
        BINARY_PREFIX, DECIMAL_PREFIX, HEXADECIMAL_PREFIX, OCTAL_PREFIX,
    };
    use crate::test::assert_warning;
    use crate::ParserIgnoreConfig;

    use super::*;

    // TODO add tests

    #[test]
    fn test_warning_trailing_zeroes() {
        let mut reader = Reader::from_str("0.00");
        let mut context = ParserContext::default();
        Number::parse(&mut reader, &mut context).expect("The parser must succeed");

        println!("{}", context.messages()[0].to_ansi_text());

        assert_warning(&context, ParserWarning::NumberWithTrailingZeroes);

        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(format!("{}0.000", prefix).as_str());
            let mut context = ParserContext::default();
            Number::parse(&mut reader, &mut context).expect("The parser must succeed");

            println!("{}", context.messages()[0].to_ansi_text());

            assert_warning(&context, ParserWarning::NumberWithTrailingZeroes);
        }
    }

    #[test]
    fn test_ignore_warning_trailing_zeroes() {
        let mut reader = Reader::from_str("0.00");
        let mut ignore = ParserIgnoreConfig::new();
        ignore.number_trailing_zeroes = true;

        let mut context = ParserContext::new(ignore);
        Number::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(context.messages().len(), 0, "There must no be messages");
    }

    #[test]
    fn test_warning_trailing_zeroes_ignores_0() {
        let mut reader = Reader::from_str("0.0");
        let mut context = ParserContext::default();
        Number::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(context.messages().len(), 0, "There must no be messages");

        for prefix in &[
            BINARY_PREFIX,
            OCTAL_PREFIX,
            DECIMAL_PREFIX,
            HEXADECIMAL_PREFIX,
        ] {
            let mut reader = Reader::from_str(format!("{}0.0", prefix).as_str());
            let mut context = ParserContext::default();
            IntegerNumber::parse(&mut reader, &mut context).expect("The parser must succeed");

            assert_eq!(context.messages().len(), 0, "There must no be messages");
        }
    }
}
