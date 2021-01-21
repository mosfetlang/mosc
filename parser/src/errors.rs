use std::fmt::Write;

use crate::io::{Cursor, Reader};

/// The error that will return all Parser methods and components in case something went wrong.
#[derive(Debug, Clone)]
pub enum ParserError {
    NotFound,

    MissingVariableNameInVariableDeclaration(Cursor),
    MissingAssignOperatorInVariableDeclaration(Cursor),
    MissingExpressionInVariableDeclaration(Cursor),

    MissingExpressionInReturnStatement(Cursor),

    ExpectedEOFInFile(Cursor),
    TwoStatementsInSameLineInFile(Cursor),
}

impl ParserError {
    // METHODS ----------------------------------------------------------------

    /// Whether the two variants are the same or not.
    pub fn variant_eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Prints the error using the specified `Reader`.
    ///
    /// # Safety
    ///
    /// This method will panic when the error is not related to `reader.
    pub fn print_error(&self, reader: &Reader) -> String {
        let reader = reader.clone();

        match self {
            ParserError::NotFound => "Not found".to_string(),
            ParserError::MissingVariableNameInVariableDeclaration(cursor) => {
                Self::print_error_at_cursor(
                    reader,
                    cursor,
                    "A name is required in the variable declaration",
                    "MissingVariableNameInVariableDeclaration",
                )
            }
            ParserError::MissingAssignOperatorInVariableDeclaration(cursor) => {
                Self::print_error_at_cursor(
                    reader,
                    cursor,
                    "An expression (= <expr>) is required in the variable declaration",
                    "MissingAssignOperatorInVariableDeclaration",
                )
            }
            ParserError::MissingExpressionInVariableDeclaration(cursor) => {
                Self::print_error_at_cursor(
                    reader,
                    cursor,
                    "An expression is required in the variable declaration",
                    "MissingExpressionInVariableDeclaration",
                )
            }
            ParserError::MissingExpressionInReturnStatement(cursor) => Self::print_error_at_cursor(
                reader,
                cursor,
                "An expression is required in the return statement",
                "MissingExpressionInReturnStatement",
            ),
            ParserError::ExpectedEOFInFile(cursor) => Self::print_error_at_cursor(
                reader,
                cursor,
                "Expected end-of-file here",
                "ExpectedEOFInFile",
            ),
            ParserError::TwoStatementsInSameLineInFile(cursor) => Self::print_error_at_cursor(
                reader,
                cursor,
                "Two or more statements in the same line are forbidden",
                "TwoStatementsInSameLineInFile",
            ),
        }
    }

    // STATIC METHODS ---------------------------------------------------------

    fn print_error_at_cursor(
        mut reader: Reader,
        cursor: &Cursor,
        message: &str,
        error_id: &str,
    ) -> String {
        reader.restore(cursor.clone());

        let span = reader.span_at_offset();
        let mut result = String::new();
        let line_num = format!("{}", span.start_cursor().line());
        let line = span.lines();

        write!(&mut result, "[error] {}\n", message).unwrap();
        write!(&mut result, "  {} | {}\n", line_num, line).unwrap();
        write!(
            &mut result,
            "  {:<line_length$} | {:<column$}^\n",
            "",
            "",
            line_length = line_num.len(),
            column = span.start_cursor().column() - 1
        )
        .unwrap();
        write!(
            &mut result,
            "  {:<line_length$} = EID: {}\n",
            "",
            error_id,
            line_length = line_num.len(),
        )
        .unwrap();

        result
    }
}
