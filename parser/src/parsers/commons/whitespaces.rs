use std::ops::RangeInclusive;

use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserContext;

// FIXME(juliotpaez): use Unicode classifications.
static WHITESPACE_CHARS: [RangeInclusive<char>; 2] = ['\t'..='\t', ' '..=' '];
// FIXME(juliotpaez): use Unicode classifications.
static MULTILINE_WHITESPACE_CHARS: [RangeInclusive<char>; 2] = ['\n'..='\n', '\r'..='\r'];

/// A valid name in the Mosfet language.
#[derive(Debug)]
pub struct Whitespace {
    span: Span,
    is_multiline: bool,
}

impl Whitespace {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Whether it is a multiline whitespaces or not.
    pub fn is_multiline(&self) -> bool {
        self.is_multiline
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an inline `Whitespace`.
    pub fn parse_inline(reader: &mut Reader, _context: &ParserContext) -> ParserResult<Whitespace> {
        cursor_manager(reader, |reader, init_cursor| {
            if let Some(_) = reader.read_many_of(&WHITESPACE_CHARS) {
                let span = reader.substring_to_current(&init_cursor);
                Ok(Whitespace {
                    span,
                    is_multiline: false,
                })
            } else {
                Err(ParserError::NotFound)
            }
        })
    }

    /// Parses a multiline `Whitespace`.
    pub fn parse_multiline(
        reader: &mut Reader,
        _context: &ParserContext,
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
                let span = reader.substring_to_current(&init_cursor);
                Ok(Whitespace { span, is_multiline })
            } else {
                Err(ParserError::NotFound)
            }
        })
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
        let mut reader = Reader::from_str("  \t\t\t  \t-rest");
        let whitespace = Whitespace::parse_inline(&mut reader, &ParserContext::default())
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
    fn test_parse_multiline_without_jump_lines() {
        let mut reader = Reader::from_str("  \t\t\t  \t-rest");
        let whitespace = Whitespace::parse_multiline(&mut reader, &ParserContext::default())
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
        let whitespace = Whitespace::parse_multiline(&mut reader, &ParserContext::default())
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
}
