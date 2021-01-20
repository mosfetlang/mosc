use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::result::ParserResult;
use crate::parsers::statements::Statement;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserContext;

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
    pub fn parse(reader: &mut Reader, context: &ParserContext) -> ParserResult<MosfetFile> {
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
                        Err(ParserError::ExpectedEOFInFile(reader.save()))
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
                            return Err(ParserError::TwoStatementsInSameLineInFile(
                                whitespace
                                    .map(|node| node.span().start_cursor().clone())
                                    .unwrap_or(reader.save()),
                            ));
                        }

                        statements.push(statement);
                    }
                    Err(ParserError::NotFound) => break,
                    Err(e) => return Err(e),
                }
            }

            // Check end.
            let span = reader.substring_to_current(&init_cursor);
            if reader.remaining_length() == 0 {
                Ok(MosfetFile { span, statements })
            } else {
                Err(ParserError::ExpectedEOFInFile(reader.save()))
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
    fn test_parse_empty() {
        let mut reader = Reader::from_str("");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            0,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_blank() {
        let mut reader = Reader::from_str("   \t \t \n\r\n    \t \t ");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            0,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_statement() {
        let mut reader = Reader::from_str(" \t  let x = 3   \n\n");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            1,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_many_statements() {
        let mut reader = Reader::from_str(" \t  let x = 3   \n let x = 3\nlet x = 3");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        assert_eq!(
            mosfet_file.statements.len(),
            3,
            "The statement length is incorrect"
        );
    }

    #[test]
    fn test_parse_err_eof_before_first_statement() {
        let mut reader = Reader::from_str(" \n t");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            matches!(mosfet_file, ParserError::ExpectedEOFInFile(..)),
            "The error is incorrect"
        );
    }

    #[test]
    fn test_parse_err_eof_after_first_statement() {
        let mut reader = Reader::from_str(" \t  let x = 3 t");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            matches!(mosfet_file, ParserError::ExpectedEOFInFile(..)),
            "The error is incorrect"
        );
    }

    #[test]
    fn test_parse_err_two_statements_same_line() {
        let mut reader = Reader::from_str("let x = 3 let y = 4");
        let mosfet_file = MosfetFile::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            matches!(mosfet_file, ParserError::TwoStatementsInSameLineInFile(..)),
            "The error is incorrect"
        );
    }
}
