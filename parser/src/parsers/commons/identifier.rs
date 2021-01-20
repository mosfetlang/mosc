use std::ops::RangeInclusive;

use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::ParserContext;

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

    /// The span of the `Identifier`.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The name of the `Identifier`.
    pub fn name(&self) -> &str {
        self.span.content()
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an `Identifier` in the specified `input` position.
    pub fn parse(reader: &mut Reader, _context: &ParserContext) -> ParserResult<Identifier> {
        let init_cursor = reader.save();
        if let None = reader.read_one_of(&HEAD_CHARS) {
            return Err(None);
        }

        reader.read_one_or_more_of(&BODY_CHARS);

        let span = reader.substring_to_current(&init_cursor);
        Ok(Identifier { span })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut reader = Reader::from_str("test-rest");
        let identifier = Identifier::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(identifier.name(), "test", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_numbers() {
        let mut reader = Reader::from_str("t3st3-rest");
        let identifier = Identifier::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(identifier.name(), "t3st3", "The name is incorrect");
    }

    #[test]
    fn test_parse_with_underscores() {
        let mut reader = Reader::from_str("_-rest");
        let identifier = Identifier::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(identifier.name(), "_", "The name is incorrect");

        let mut reader = Reader::from_str("___test___32___-rest");
        let identifier = Identifier::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            identifier.name(),
            "___test___32___",
            "The name is incorrect"
        );
    }
}
