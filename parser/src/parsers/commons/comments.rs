use std::sync::Arc;

use doclog::Color;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::result::ParserResult;
use crate::parsers::utils::{cursor_manager, generate_error_log, generate_source_code};
use crate::parsers::ParserResultError;
use crate::{ParserError, ParserNode};

pub static SINGLE_LINE_COMMENT_TOKEN: &str = "# ";
pub static MULTILINE_COMMENT_TOKEN: &str = "#";
pub static MULTILINE_COMMENT_REPEAT_TOKEN: &str = "+";

/// A valid comment in the Mosfet language.
#[derive(Debug)]
pub struct Comment {
    span: Arc<Span>,
    is_multiline_type: bool,
    message: Arc<Span>,
    repeated_tokens: usize,
}

impl Comment {
    // GETTERS ----------------------------------------------------------------

    /// Whether the comment type is multiline '#+...+#' or not '# ...'.
    pub fn is_multiline_type(&self) -> bool {
        self.is_multiline_type
    }

    pub fn message(&self) -> &Arc<Span> {
        &self.message
    }

    pub fn repeated_tokens(&self) -> &usize {
        &self.repeated_tokens
    }

    pub fn immediately_closed(&self) -> bool {
        self.is_multiline_type && self.message.content().is_empty()
    }

    /// Whether the comment is placed in multiple lines or not, e.g. '#+...\n...+#'
    pub fn is_multiline(&self) -> bool {
        self.message.start_cursor().line() != self.message.end_cursor().line()
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an inline `Comment`.
    pub fn parse_inline(
        reader: &mut Reader,
        _context: &mut ParserContext,
    ) -> ParserResult<Comment> {
        cursor_manager(reader, |reader, init_cursor| {
            if !reader.read(SINGLE_LINE_COMMENT_TOKEN) {
                return Err(ParserResultError::NotFound);
            }

            let init_message_cursor = reader.save_cursor();
            let _ = reader.read_until("\n", true);

            Ok(Comment {
                span: Arc::new(reader.substring_to_current(&init_cursor)),
                is_multiline_type: false,
                message: Arc::new(reader.substring_to_current(&init_message_cursor)),
                repeated_tokens: 0,
            })
        })
    }

    /// Parses a multiline `Comment`.
    pub fn parse_multiline(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<Comment> {
        cursor_manager(reader, |reader, init_cursor| {
            if !reader.read(MULTILINE_COMMENT_TOKEN) {
                return Err(ParserResultError::NotFound);
            }

            // Read opening tokens to build close ones.
            let mut close_token = String::new();
            loop {
                if !reader.read(MULTILINE_COMMENT_REPEAT_TOKEN) {
                    if close_token.is_empty() {
                        return Err(ParserResultError::NotFound);
                    }

                    break;
                }

                close_token.push_str(MULTILINE_COMMENT_REPEAT_TOKEN);
            }

            close_token.push_str(MULTILINE_COMMENT_TOKEN);

            let init_message_cursor = reader.save_cursor();

            // Case when the comment is opened and closed immediately, e.g. #+#, #++#, ...
            if reader.read(MULTILINE_COMMENT_TOKEN) {
                return Ok(Comment {
                    span: Arc::new(reader.substring_to_current(&init_cursor)),
                    is_multiline_type: true,
                    message: Arc::new(reader.substring(&init_message_cursor, &init_message_cursor)),
                    repeated_tokens: close_token.len() - 1,
                });
            }

            if reader.read_until(close_token.as_str(), false).is_none() {
                context.add_message(generate_error_log(
                    ParserError::MultilineCommentWithoutEndToken,
                    format!(
                        "The end token '{}' was expected here to close the multiline comment",
                        close_token
                    ),
                    |log| {
                        generate_source_code(log, &reader, |doc| {
                            doc.highlight_section(
                                init_cursor.byte_offset()..reader.byte_offset(),
                                Some(Color::Magenta),
                            )
                            .highlight_cursor_message(
                                reader.byte_offset(),
                                format!("Insert here the close token '{}'", close_token),
                                None,
                            )
                            .related_document(|doc| {
                                let end_position = reader.content().len();
                                doc.title(arcstr::literal!("or"))
                                    .highlight_section(
                                        init_cursor.byte_offset()..end_position,
                                        Some(Color::Magenta),
                                    )
                                    .highlight_cursor_message(
                                        end_position,
                                        format!("Insert here the close token '{}'", close_token),
                                        None,
                                    )
                            })
                        })
                    },
                ));

                return Err(ParserResultError::Error);
            }

            let end_message_cursor = reader.save_cursor();
            assert!(reader.read(close_token.as_str()));

            Ok(Comment {
                span: Arc::new(reader.substring_to_current(&init_cursor)),
                is_multiline_type: true,
                message: Arc::new(reader.substring(&init_message_cursor, &end_message_cursor)),
                repeated_tokens: close_token.len() - 1,
            })
        })
    }
}

impl ParserNode for Comment {
    fn span(&self) -> &Arc<Span> {
        &self.span
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::test::{assert_error, assert_not_found};

    use super::*;

    #[test]
    fn test_parse_inline() {
        let mut reader = Reader::from_content(arcstr::literal!("# This is a comment\n"));
        let mut context = ParserContext::default();
        let comment =
            Comment::parse_inline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            comment.span.content(),
            "# This is a comment",
            "The content is incorrect"
        );
        assert_eq!(
            comment.is_multiline_type, false,
            "The is_multiline_type is incorrect"
        );
        assert_eq!(
            comment.message.content(),
            "This is a comment",
            "The message is incorrect"
        );
        assert_eq!(
            comment.repeated_tokens, 0,
            "The repeated_tokens is incorrect"
        );
    }

    #[test]
    fn test_parse_inline_till_end() {
        let mut reader = Reader::from_content(arcstr::literal!("# This is a comment"));
        let mut context = ParserContext::default();
        let comment =
            Comment::parse_inline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            comment.span.content(),
            "# This is a comment",
            "The content is incorrect"
        );
        assert_eq!(
            comment.is_multiline_type, false,
            "The is_multiline_type is incorrect"
        );
        assert_eq!(
            comment.message.content(),
            "This is a comment",
            "The message is incorrect"
        );
        assert_eq!(
            comment.repeated_tokens, 0,
            "The repeated_tokens is incorrect"
        );
    }

    #[test]
    fn test_parse_inline_not_found() {
        for content in &["", "#", "#This is a comment"] {
            let mut reader = Reader::from_content(*content);
            let mut context = ParserContext::default();
            let error = Comment::parse_inline(&mut reader, &mut context)
                .expect_err("The parser must not succeed");

            assert_not_found(&context, &error, 0);
        }
    }

    #[test]
    fn test_parse_multiline() {
        let mut reader = Reader::from_content(arcstr::literal!("#+This is a\n # + comment+#"));
        let mut context = ParserContext::default();
        let comment =
            Comment::parse_multiline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            comment.span.content(),
            "#+This is a\n # + comment+#",
            "The content is incorrect"
        );
        assert_eq!(
            comment.is_multiline_type, true,
            "The is_multiline_type is incorrect"
        );
        assert_eq!(
            comment.message.content(),
            "This is a\n # + comment",
            "The message is incorrect"
        );
        assert_eq!(
            comment.repeated_tokens, 1,
            "The repeated_tokens is incorrect"
        );
    }

    #[test]
    fn test_parse_multiline_many_tokens() {
        let mut reader =
            Reader::from_content(arcstr::literal!("#+++This is a ++# +# # + comment++++#"));
        let mut context = ParserContext::default();
        let comment =
            Comment::parse_multiline(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            comment.span.content(),
            "#+++This is a ++# +# # + comment++++#",
            "The content is incorrect"
        );
        assert_eq!(
            comment.is_multiline_type, true,
            "The is_multiline_type is incorrect"
        );
        assert_eq!(
            comment.message.content(),
            "This is a ++# +# # + comment+",
            "The message is incorrect"
        );
        assert_eq!(
            comment.repeated_tokens, 3,
            "The repeated_tokens is incorrect"
        );
    }

    #[test]
    fn test_parse_multiline_immediately_closed() {
        for content in &["#+#", "#++#", "#+++#"] {
            let mut reader = Reader::from_content(*content);
            let mut context = ParserContext::default();
            let comment = Comment::parse_multiline(&mut reader, &mut context)
                .expect("The parser must succeed");

            assert_eq!(comment.span.content(), *content, "The content is incorrect");
            assert_eq!(
                comment.is_multiline_type, true,
                "The is_multiline_type is incorrect"
            );
            assert_eq!(comment.message.content(), "", "The message is incorrect");
            assert_eq!(
                comment.repeated_tokens,
                content.len() - 2,
                "The repeated_tokens is incorrect"
            );
        }
    }

    #[test]
    fn test_parse_multiline_not_found() {
        for content in &["", "#", "#This is a comment"] {
            let mut reader = Reader::from_content(*content);
            let mut context = ParserContext::default();
            let error = Comment::parse_multiline(&mut reader, &mut context)
                .expect_err("The parser must not succeed");

            assert_not_found(&context, &error, 0);
        }
    }

    #[test]
    fn test_parse_multiline_err_without_end_token() {
        let mut reader = Reader::from_content(arcstr::literal!("#++ This is a comment"));
        let mut context = ParserContext::default();
        let error = Comment::parse_multiline(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MultilineCommentWithoutEndToken,
        );
    }
}
