use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserResultError;
use crate::ParserNode;

// FIXME(juliotpaez): use Unicode classifications.
pub static HEAD_CHARS: [RangeInclusive<char>; 3] = ['A'..='Z', '_'..='_', 'a'..='z'];
// FIXME(juliotpaez): use Unicode classifications.
pub static BODY_CHARS: [RangeInclusive<char>; 4] = ['0'..='9', 'A'..='Z', '_'..='_', 'a'..='z'];

/// A valid name in the Mosfet language.
#[derive(Debug)]
pub struct Identifier {
    span: Arc<Span>,
}

impl Identifier {
    // GETTERS ----------------------------------------------------------------

    pub fn content(&self) -> &str {
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

            let span = Arc::new(reader.substring_to_current(&init_cursor));
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

        if id.content() == keyword {
            true
        } else {
            reader.restore(init_cursor);
            false
        }
    }
}

impl ParserNode for Identifier {
    fn span(&self) -> &Arc<Span> {
        &self.span
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::test::assert_not_found;

    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut reader = Reader::from_content(arcstr::literal!("test-rest"));
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.content(), "test", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_numbers() {
        let mut reader = Reader::from_content(arcstr::literal!("t3st3-rest"));
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.content(), "t3st3", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_underscores() {
        let mut reader = Reader::from_content(arcstr::literal!("_-rest"));
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(identifier.content(), "_", "The name is incorrect");

        let mut reader = Reader::from_content(arcstr::literal!("___test___32___-rest"));
        let mut context = ParserContext::default();
        let identifier =
            Identifier::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            identifier.content(),
            "___test___32___",
            "The name is incorrect"
        );
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_content(arcstr::literal!("23test"));
        let mut context = ParserContext::default();
        let error =
            Identifier::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_not_found(&context, &error, 0);
    }

    #[test]
    fn test_parse_keyword() {
        let mut reader = Reader::from_content(arcstr::literal!("let me test it"));
        let mut context = ParserContext::default();
        let result = Identifier::parse_keyword(&mut reader, &mut context, "let");

        assert_eq!(result, true, "The result is incorrect");
    }

    #[test]
    fn test_parse_keyword_err() {
        let mut reader = Reader::from_content(arcstr::literal!("letting me test it"));
        let mut context = ParserContext::default();
        let result = Identifier::parse_keyword(&mut reader, &mut context, "let");

        assert_eq!(result, false, "The result is incorrect");
    }
}
