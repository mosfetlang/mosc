use std::ops::RangeInclusive;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserResultError;

// FIXME(juliotpaez): use Unicode classifications.
static HEAD_CHARS: [RangeInclusive<char>; 3] = ['A'..='Z', '_'..='_', 'a'..='z'];
// FIXME(juliotpaez): use Unicode classifications.
static BODY_CHARS: [RangeInclusive<char>; 4] = ['0'..='9', 'A'..='Z', '_'..='_', 'a'..='z'];

/// A valid name in the Mosfet language.
#[derive(Debug)]
pub struct Identifier {
    span: Span,
}

impl Identifier {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The name of the `Identifier`.
    pub fn name(&self) -> &str {
        self.span.content()
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an `Identifier`.
    pub fn parse(reader: &mut Reader, _context: &mut ParserContext) -> ParserResult<Identifier> {
        cursor_manager(reader, |reader, init_cursor| {
            if let None = reader.read_one_of(&HEAD_CHARS) {
                return Err(ParserResultError::NotFound);
            }

            reader.read_many_of(&BODY_CHARS);

            let span = reader.substring_to_current(&init_cursor);
            Ok(Identifier { span })
        })
    }

    /// Parses a keyword.
    pub fn parse_keyword(reader: &mut Reader, _context: &mut ParserContext, keyword: &str) -> bool {
        let init_cursor = reader.save_cursor();
        let id = match Identifier::parse(reader, _context) {
            Ok(v) => v,
            Err(_) => {
                return false;
            }
        };

        if id.name() == keyword {
            true
        } else {
            reader.restore(init_cursor);
            false
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::parsers::ParserResultError;

    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut reader = Reader::from_str("test-rest");
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.name(), "test", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_numbers() {
        let mut reader = Reader::from_str("t3st3-rest");
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.name(), "t3st3", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_underscores() {
        let mut reader = Reader::from_str("_-rest");
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.name(), "_", "The name is incorrect");

        let mut reader = Reader::from_str("___test___32___-rest");
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            identifier.name(),
            "___test___32___",
            "The name is incorrect"
        );
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("23test");
        let mut context = ParserContext::default();
        let error =
            Identifier::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_eq!(error, ParserResultError::NotFound, "The error is incorrect");
        assert_eq!(reader.offset(), 0, "The offset is incorrect");
    }

    #[test]
    fn test_parse_keyword() {
        let mut reader = Reader::from_str("let me test it");
        let mut context = ParserContext::default();
        let result = Identifier::parse_keyword(&mut reader, &mut context, "let");

        assert_eq!(result, true, "The result is incorrect");
    }

    #[test]
    fn test_parse_keyword_err() {
        let mut reader = Reader::from_str("letting me test it");
        let mut context = ParserContext::default();
        let result = Identifier::parse_keyword(&mut reader, &mut context, "let");

        assert_eq!(result, false, "The result is incorrect");
    }
}
