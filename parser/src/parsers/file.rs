use doclog::Color;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::result::ParserResult;
use crate::parsers::statements::Statement;
use crate::parsers::utils::{cursor_manager, generate_error_log, generate_source_code};
use crate::parsers::ParserResultError;
use crate::ParserError;

/// A Mosfet file.
#[derive(Debug)]
pub struct MosfetFile {
    span: Span,
    statements: Vec<Statement>,
}

impl MosfetFile {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The statements of the file.
    pub fn statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a Mosfet file.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<MosfetFile> {
        cursor_manager(reader, |reader, init_cursor| {
            let mut statements = Vec::new();

            // First statement.
            let _ = Whitespace::parse_multiline(reader, context);

            match Statement::parse(reader, context) {
                Ok(statement) => statements.push(statement),
                Err(_) => {
                    // Check end.
                    let span = reader.substring_to_current(&init_cursor);
                    return if reader.remaining_length() == 0 {
                        Ok(MosfetFile { span, statements })
                    } else {
                        context.add_message(generate_error_log(
                            ParserError::NotAMosfetFile,
                            "The file is not recognized as valid Mosfet file".to_string(),
                            |log| log,
                        ));

                        Err(ParserResultError::Error)
                    };
                }
            }

            // Next statements.
            loop {
                let whitespace = Whitespace::parse_multiline(reader, context);

                match Statement::parse(reader, context) {
                    Ok(statement) => {
                        // Check whitespace is multiline to prevent two statements in the same line.
                        if !whitespace
                            .as_ref()
                            .map(|ws| ws.is_multiline())
                            .unwrap_or(false)
                        {
                            context.add_message(generate_error_log(
                                ParserError::TwoStatementsInSameLineInFile,
                                "Two statements in the same line are forbidden".to_string(),
                                |log| {
                                    generate_source_code(log, &reader, |doc| {
                                        doc.highlight_cursor_str(
                                            statements
                                                .last()
                                                .unwrap()
                                                .span()
                                                .end_cursor()
                                                .byte_offset(),
                                            Some("Insert a new line (\\n) here"),
                                            None,
                                        )
                                    })
                                },
                            ));

                            return Err(ParserResultError::Error);
                        }

                        statements.push(statement);
                    }
                    Err(ParserResultError::NotFound) => break,
                    Err(ParserResultError::Error) => return Err(ParserResultError::Error),
                }
            }

            // Check end.
            let span = reader.substring_to_current(&init_cursor);
            if reader.remaining_length() == 0 {
                Ok(MosfetFile { span, statements })
            } else {
                context.add_message(generate_error_log(
                    ParserError::ExpectedEOFInFile,
                    "The End Of File (EOF) was expected here".to_string(),
                    |log| {
                        let last_statement = statements.last().unwrap();
                        generate_source_code(log, &reader, |doc| {
                            let doc = doc.highlight_cursor_str(
                                last_statement.span().end_cursor().byte_offset(),
                                Some("The file must end here"),
                                None,
                            );

                            if reader.content().len() - reader.byte_offset() != 0 {
                                doc.highlight_section_str(
                                    last_statement.span().end_cursor().byte_offset()
                                        ..reader.content().len(),
                                    Some("Remove this code"),
                                    Some(Color::Magenta),
                                )
                            } else {
                                doc
                            }
                        })
                    },
                ));

                Err(ParserResultError::Error)
            }
        })
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::test::assert_error;
    use crate::ParserError;

    use super::*;

    #[test]
    fn test_parse_empty() {
        let mut reader = Reader::from_str("");
        let mut context = ParserContext::default();
        let mosfet_file =
            MosfetFile::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            0,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_blank() {
        let mut reader = Reader::from_str("   \t \t \n\r\n    \t \t ");
        let mut context = ParserContext::default();
        let mosfet_file =
            MosfetFile::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            0,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_statement() {
        let mut reader = Reader::from_str(" \t  let x = 3   \n\n");
        let mut context = ParserContext::default();
        let mosfet_file =
            MosfetFile::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            1,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_many_statements() {
        let mut reader = Reader::from_str(" \t  let x = 3   \n let x = 3\nlet x = 3");
        let mut context = ParserContext::default();
        let mosfet_file =
            MosfetFile::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            3,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_err_eof_before_first_statement() {
        let mut reader = Reader::from_str(" \n t");
        let mut context = ParserContext::default();
        let error =
            MosfetFile::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_error(&context, &error, ParserError::NotAMosfetFile);
    }

    #[test]
    fn test_parse_err_eof_after_first_statement() {
        let mut reader = Reader::from_str("let x = 3 t");
        let mut context = ParserContext::default();
        let error =
            MosfetFile::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_error(&context, &error, ParserError::ExpectedEOFInFile);
    }

    #[test]
    fn test_parse_err_two_statements_same_line() {
        let mut reader = Reader::from_str("let x = 3 let y = 4");
        let mut context = ParserContext::default();
        let error =
            MosfetFile::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_error(&context, &error, ParserError::TwoStatementsInSameLineInFile);
    }
}
