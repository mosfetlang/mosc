use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::comments::Comment;
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
    elements: Vec<WhitespaceElement>,
}

/// An element of the whitespace.
#[derive(Debug)]
pub enum WhitespaceElement {
    Whitespace(Arc<Span>),
    Comment(Arc<Comment>),
}

impl Whitespace {
    // GETTERS ----------------------------------------------------------------

    /// Whether the node contains a new line character or not.
    pub fn is_multiline(&self) -> bool {
        self.is_multiline
    }

    pub fn elements(&self) -> &Vec<WhitespaceElement> {
        &self.elements
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an inline `Whitespace`.
    pub fn parse_inline(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<Whitespace> {
        cursor_manager(reader, |reader, init_cursor| {
            let mut is_multiline = false;
            let mut elements = Vec::new();

            loop {
                let pre_cursor = reader.save_cursor();

                if reader.read_many_of(&WHITESPACE_CHARS).is_some() {
                    let span = Arc::new(reader.substring_to_current(&pre_cursor));
                    elements.push(WhitespaceElement::Whitespace(span));

                    continue;
                }

                match Comment::parse_multiline(reader, context) {
                    Ok(comment) => {
                        is_multiline |= comment.is_multiline();
                        elements.push(WhitespaceElement::Comment(Arc::new(comment)));

                        continue;
                    }
                    Err(ParserResultError::NotFound) => { /* ignore */ }
                    Err(ParserResultError::Error) => return Err(ParserResultError::Error),
                }

                break;
            }

            if elements.is_empty() {
                Err(ParserResultError::NotFound)
            } else {
                let span = Arc::new(reader.substring_to_current(&init_cursor));
                Ok(Whitespace {
                    span,
                    is_multiline,
                    elements,
                })
            }
        })
    }

    /// Parses a multiline `Whitespace`.
    pub fn parse_multiline(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<Whitespace> {
        cursor_manager(reader, |reader, init_cursor| {
            let mut is_multiline = false;
            let mut elements = Vec::new();

            loop {
                let pre_cursor = reader.save_cursor();

                let mut any_whitespace = false;
                loop {
                    if reader.read_many_of(&WHITESPACE_CHARS).is_some() {
                        any_whitespace = true;
                        continue;
                    }

                    if reader.read_many_of(&MULTILINE_WHITESPACE_CHARS).is_some() {
                        any_whitespace = true;
                        is_multiline = true;
                        continue;
                    }

                    break;
                }

                if any_whitespace {
                    let span = Arc::new(reader.substring_to_current(&pre_cursor));
                    elements.push(WhitespaceElement::Whitespace(span));
                }

                match Comment::parse_inline(reader, context) {
                    Ok(comment) => {
                        elements.push(WhitespaceElement::Comment(Arc::new(comment)));

                        continue;
                    }
                    Err(ParserResultError::NotFound) => { /* ignore */ }
                    Err(ParserResultError::Error) => return Err(ParserResultError::Error),
                }

                match Comment::parse_multiline(reader, context) {
                    Ok(comment) => {
                        is_multiline |= comment.is_multiline();
                        elements.push(WhitespaceElement::Comment(Arc::new(comment)));

                        continue;
                    }
                    Err(ParserResultError::NotFound) => { /* ignore */ }
                    Err(ParserResultError::Error) => return Err(ParserResultError::Error),
                }

                break;
            }

            if elements.is_empty() {
                Err(ParserResultError::NotFound)
            } else {
                let span = Arc::new(reader.substring_to_current(&init_cursor));
                Ok(Whitespace {
                    span,
                    is_multiline,
                    elements,
                })
            }
        })
    }

    /// Parses an inline `Whitespace` or returns an empty one.
    pub fn parse_inline_or_default(reader: &mut Reader, context: &mut ParserContext) -> Whitespace {
        Self::parse_inline(reader, context).unwrap_or(Whitespace {
            span: Arc::new(reader.substring_to_current(&reader.save_cursor())),
            is_multiline: false,
            elements: Vec::new(),
        })
    }

    /// Parses a multiline `Whitespace` or returns an empty one.
    pub fn parse_multiline_or_default(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> Whitespace {
        Self::parse_multiline(reader, context).unwrap_or(Whitespace {
            span: Arc::new(reader.substring_to_current(&reader.save_cursor())),
            is_multiline: false,
            elements: Vec::new(),
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
        let mut reader = Reader::from_content(arcstr::literal!("  \t\t\t  \t\n"));
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
        assert_eq!(
            whitespace.elements.len(),
            1,
            "The elements.len is incorrect"
        );

        match &whitespace.elements[0] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "  \t\t\t  \t", "The element is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
        }
    }

    #[test]
    fn test_parse_inline_exhaustive() {
        for char_range in &WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_content(text.as_str().into());
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
                assert_eq!(
                    whitespace.elements.len(),
                    1,
                    "The elements.len is incorrect"
                );

                match &whitespace.elements[0] {
                    WhitespaceElement::Whitespace(v) => {
                        assert_eq!(v.content(), text.as_str(), "The element is incorrect");
                    }
                    WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
                }
            }
        }
    }

    #[test]
    fn test_parse_inline_with_comments() {
        let mut reader =
            Reader::from_content(arcstr::literal!("  #+multiline\ncomment+##++#  test"));
        let mut context = ParserContext::default();
        let whitespace =
            Whitespace::parse_inline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            whitespace.span.content(),
            "  #+multiline\ncomment+##++#  ",
            "The content is incorrect"
        );
        assert_eq!(
            whitespace.is_multiline, true,
            "The is_multiline is incorrect"
        );
        assert_eq!(
            whitespace.elements.len(),
            4,
            "The elements.len is incorrect"
        );

        match &whitespace.elements[0] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "  ", "The element[0] is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type for 0"),
        }

        match &whitespace.elements[1] {
            WhitespaceElement::Whitespace(_) => panic!("Incorrect element type for 1"),
            WhitespaceElement::Comment(v) => {
                assert_eq!(
                    v.content(),
                    "#+multiline\ncomment+#",
                    "The element[1] is incorrect"
                );
            }
        }

        match &whitespace.elements[2] {
            WhitespaceElement::Whitespace(_) => panic!("Incorrect element type for 2"),
            WhitespaceElement::Comment(v) => {
                assert_eq!(v.content(), "#++#", "The element[1] is incorrect");
            }
        }

        match &whitespace.elements[3] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "  ", "The element[3] is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type for 3"),
        }
    }

    #[test]
    fn test_parse_multiline_without_jump_lines() {
        let mut reader = Reader::from_content(arcstr::literal!("  \t\t\t  \t-rest"));
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
        assert_eq!(
            whitespace.elements.len(),
            1,
            "The elements.len is incorrect"
        );

        match &whitespace.elements[0] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "  \t\t\t  \t", "The element is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
        }
    }

    #[test]
    fn test_parse_multiline_with_jump_lines() {
        let mut reader = Reader::from_content(arcstr::literal!("\n\n \r\n \t\t\n\t \r \t-rest"));
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
        assert_eq!(
            whitespace.elements.len(),
            1,
            "The elements.len is incorrect"
        );

        match &whitespace.elements[0] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(
                    v.content(),
                    "\n\n \r\n \t\t\n\t \r \t",
                    "The element is incorrect"
                );
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
        }
    }

    #[test]
    fn test_parse_multiline_exhaustive() {
        for char_range in &WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_content(text.as_str().into());
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
                assert_eq!(
                    whitespace.elements.len(),
                    1,
                    "The elements.len is incorrect"
                );

                match &whitespace.elements[0] {
                    WhitespaceElement::Whitespace(v) => {
                        assert_eq!(v.content(), text.as_str(), "The element is incorrect");
                    }
                    WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
                }
            }
        }

        for char_range in &MULTILINE_WHITESPACE_CHARS {
            for char in char_range.clone() {
                let text = format!("{}", char);
                let mut reader = Reader::from_content(text.as_str().into());
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
                assert_eq!(
                    whitespace.elements.len(),
                    1,
                    "The elements.len is incorrect"
                );

                match &whitespace.elements[0] {
                    WhitespaceElement::Whitespace(v) => {
                        assert_eq!(v.content(), text.as_str(), "The element is incorrect");
                    }
                    WhitespaceElement::Comment(_) => panic!("Incorrect element type"),
                }
            }
        }
    }

    #[test]
    fn test_parse_multiline_with_comments() {
        let mut reader =
            Reader::from_content(arcstr::literal!("  #+multiline\ncomment+## test\n x"));
        let mut context = ParserContext::default();
        let whitespace = Whitespace::parse_multiline(&mut reader, &mut context)
            .expect("The parser must succeed");

        assert_eq!(
            whitespace.span.content(),
            "  #+multiline\ncomment+## test\n ",
            "The content is incorrect"
        );
        assert_eq!(
            whitespace.is_multiline, true,
            "The is_multiline is incorrect"
        );
        assert_eq!(
            whitespace.elements.len(),
            4,
            "The elements.len is incorrect"
        );

        match &whitespace.elements[0] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "  ", "The element[0] is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type for 0"),
        }

        match &whitespace.elements[1] {
            WhitespaceElement::Whitespace(_) => panic!("Incorrect element type for 1"),
            WhitespaceElement::Comment(v) => {
                assert_eq!(
                    v.content(),
                    "#+multiline\ncomment+#",
                    "The element[1] is incorrect"
                );
            }
        }

        match &whitespace.elements[2] {
            WhitespaceElement::Whitespace(_) => panic!("Incorrect element type for 2"),
            WhitespaceElement::Comment(v) => {
                assert_eq!(v.content(), "# test", "The element[1] is incorrect");
            }
        }

        match &whitespace.elements[3] {
            WhitespaceElement::Whitespace(v) => {
                assert_eq!(v.content(), "\n ", "The element[3] is incorrect");
            }
            WhitespaceElement::Comment(_) => panic!("Incorrect element type for 3"),
        }
    }
}
