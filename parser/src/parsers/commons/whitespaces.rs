use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserResultError;
use crate::ParserNode;

// Follow UCD specification: https://www.unicode.org/Public/13.0.0/ucd/PropList.txt
pub static WHITESPACE_CHARS: [RangeInclusive<char>; 8] = [
    '\u{9}'..='\u{9}',
    '\u{20}'..='\u{20}',
    '\u{A0}'..='\u{A0}',
    '\u{1680}'..='\u{1680}',
    '\u{2000}'..='\u{200A}',
    '\u{202F}'..='\u{202F}',
    '\u{205F}'..='\u{205F}',
    '\u{3000}'..='\u{3000}',
];
// Follow UCD specification: https://www.unicode.org/Public/13.0.0/ucd/PropList.txt
pub static MULTILINE_WHITESPACE_CHARS: [RangeInclusive<char>; 3] = [
    '\u{A}'..='\u{D}',
    '\u{85}'..='\u{85}',
    '\u{2028}'..='\u{2029}',
];

/// A valid name in the Mosfet language.
#[derive(Debug)]
pub struct Whitespace {
    span: Arc<Span>,
    is_multiline: bool,
}

impl Whitespace {
    // GETTERS ----------------------------------------------------------------

    /// Whether it is a multiline whitespaces or not.
    pub fn is_multiline(&self) -> bool {
        self.is_multiline
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an inline `Whitespace`.
    pub fn parse_inline(
        reader: &mut Reader,
        _context: &mut ParserContext,
    ) -> ParserResult<Whitespace> {
        cursor_manager(reader, |reader, init_cursor| {
            if let Some(_) = reader.read_many_of(&WHITESPACE_CHARS) {
                let span = Arc::new(reader.substring_to_current(&init_cursor));
                Ok(Whitespace {
                    span,
                    is_multiline: false,
                })
            } else {
                Err(ParserResultError::NotFound)
            }
        })
    }

    /// Parses a multiline `Whitespace`.
    pub fn parse_multiline(
        reader: &mut Reader,
        _context: &mut ParserContext,
    ) -> ParserResult<Whitespace> {
        cursor_manager(reader, |reader, init_cursor| {
            let mut any = false;
            let mut is_multiline = false;
            loop {
                match reader.read_many_of(&WHITESPACE_CHARS) {
                    Some(_) => {
                        any = true;
                    }
                    None => match reader.read_many_of(&MULTILINE_WHITESPACE_CHARS) {
                        Some(_) => {
                            any = true;
                            is_multiline = true;
                        }
                        None => {
                            break;
                        }
                    },
                }
            }

            if any {
                let span = Arc::new(reader.substring_to_current(&init_cursor));
                Ok(Whitespace { span, is_multiline })
            } else {
                Err(ParserResultError::NotFound)
            }
        })
    }
}

impl ParserNode for Whitespace {
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
    fn test_parse_inline() {
        let mut reader = Reader::from_str("  \t\t\t  \t\n");
        let mut context = ParserContext::default();
        let whitespace =
            Whitespace::parse_inline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            whitespace.span.content(),
            "  \t\t\t  \t",
            "The content is incorrect"
        );
        assert_eq!(
            whitespace.is_multiline, false,
            "The is_multiline is incorrect"
        );
    }

    #[test]
    fn test_parse_inline_exhaustive() {
        for char_range in &WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_str(text.as_str());
                let mut context = ParserContext::default();
                let whitespace = Whitespace::parse_inline(&mut reader, &mut context)
                    .expect("The parser must succeed");

                assert_eq!(
                    whitespace.span.content(),
                    text.as_str(),
                    "The content is incorrect"
                );
                assert_eq!(
                    whitespace.is_multiline, false,
                    "The is_multiline is incorrect"
                );
            }
        }
    }

    #[test]
    fn test_parse_multiline_without_jump_lines() {
        let mut reader = Reader::from_str("  \t\t\t  \t-rest");
        let mut context = ParserContext::default();
        let whitespace = Whitespace::parse_multiline(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            whitespace.span.content(),
            "  \t\t\t  \t",
            "The content is incorrect"
        );
        assert_eq!(
            whitespace.is_multiline, false,
            "The is_multiline is incorrect"
        );
    }

    #[test]
    fn test_parse_multiline_with_jump_lines() {
        let mut reader = Reader::from_str("\n\n \r\n \t\t\n\t \r \t-rest");
        let mut context = ParserContext::default();
        let whitespace = Whitespace::parse_multiline(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            whitespace.span.content(),
            "\n\n \r\n \t\t\n\t \r \t",
            "The content is incorrect"
        );
        assert_eq!(
            whitespace.is_multiline, true,
            "The is_multiline is incorrect"
        );
    }

    #[test]
    fn test_parse_multiline_exhaustive() {
        for char_range in &WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_str(text.as_str());
                let mut context = ParserContext::default();
                let whitespace = Whitespace::parse_multiline(&mut reader, &mut context)
                    .expect("The parser must succeed");

                assert_eq!(
                    whitespace.span.content(),
                    text.as_str(),
                    "The content is incorrect"
                );
                assert_eq!(
                    whitespace.is_multiline, false,
                    "The is_multiline is incorrect"
                );
            }
        }

        for char_range in &MULTILINE_WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_str(text.as_str());
                let mut context = ParserContext::default();
                let whitespace = Whitespace::parse_multiline(&mut reader, &mut context)
                    .expect("The parser must succeed");

                assert_eq!(
                    whitespace.span.content(),
                    text.as_str(),
                    "The content is incorrect"
                );
                assert_eq!(
                    whitespace.is_multiline, true,
                    "The is_multiline is incorrect"
                );
            }
        }
    }
}
