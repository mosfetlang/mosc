use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::expressions::Expression;
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserResultError;

static KEYWORD: &str = "return";

/// A return statement with a compulsory expression.
#[derive(Debug)]
pub struct ReturnStatement {
    span: Span,
    expression: Expression,
}

impl ReturnStatement {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The expression of the return statement.
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a return statement.
    pub fn parse(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<ReturnStatement> {
        cursor_manager(reader, |reader, init_cursor| {
            if !Identifier::parse_keyword(reader, context, KEYWORD) {
                return Err(ParserResultError::NotFound);
            }

            let whitespace = Whitespace::parse_inline(reader, context);

            let expression = match Expression::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    // TODO improve with a log.
                    return Err(ParserResultError::Error);
                    // return Err(ParserError::MissingExpressionInReturnStatement(
                    //     whitespace
                    //         .map(|node| node.span().start_cursor().clone())
                    //         .unwrap_or(reader.save()),
                    // ));
                }
            };

            let span = reader.substring_to_current(&init_cursor);
            Ok(ReturnStatement { span, expression })
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
    fn test_parse() {
        // With whitespaces.
        let mut reader = Reader::from_str("return    test");
        let mut context = ParserContext::default();
        let statement =
            ReturnStatement::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Expression::VariableAccess(identifier) = statement.expression {
            assert_eq!(identifier.name(), "test", "The literal access is incorrect");
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let mut context = ParserContext::default();
        let error = ReturnStatement::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_eq!(error, ParserResultError::NotFound, "The error is incorrect");
        assert_eq!(reader.offset(), 0, "The offset is incorrect");
    }

    #[test]
    fn test_parse_err_missing_expression() {
        let mut reader = Reader::from_str("return");
        let mut context = ParserContext::default();
        let error = ReturnStatement::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MissingExpressionInReturnStatement,
        );
    }
}
