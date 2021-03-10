use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::expressions::Expression;
use crate::parsers::result::ParserResult;
use crate::parsers::utils::{cursor_manager, generate_error_log, generate_source_code};
use crate::parsers::ParserResultError;
use crate::{ParserError, ParserNode};

static KEYWORD: &str = "return";

/// A return statement with a compulsory expression.
#[derive(Debug)]
pub struct ReturnStatement {
    span: Arc<Span>,
    expression: Arc<Expression>,
    pre_expression_whitespace: Arc<Whitespace>,
}

impl ReturnStatement {
    // GETTERS ----------------------------------------------------------------

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn pre_expression_whitespace(&self) -> &Arc<Whitespace> {
        &self.pre_expression_whitespace
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

            let pre_expression_whitespace = Whitespace::parse_multiline_or_default(reader, context);

            let expression = match Expression::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    context.add_message(generate_error_log(
                        ParserError::MissingExpressionInReturnStatement,
                        "An expression was expected to specify the value to return".to_string(),
                        |log| {
                            generate_source_code(log, &reader, |doc| {
                                doc.highlight_cursor_str(
                                    reader.offset(),
                                    Some("Insert an expression here"),
                                    None,
                                )
                            })
                        },
                    ));

                    return Err(ParserResultError::Error);
                }
            };

            let span = Arc::new(reader.substring_to_current(&init_cursor));
            Ok(ReturnStatement {
                span,
                expression: Arc::new(expression),
                pre_expression_whitespace: Arc::new(pre_expression_whitespace),
            })
        })
    }
}

impl ParserNode for ReturnStatement {
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
    use crate::ParserError;

    use super::*;

    #[test]
    fn test_parse() {
        // With whitespaces.
        let mut reader = Reader::from_str("return    test");
        let mut context = ParserContext::default();
        let statement =
            ReturnStatement::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Expression::VariableAccess(identifier) = statement.expression.as_ref() {
            assert_eq!(
                identifier.content(),
                "test",
                "The literal access is incorrect"
            );
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

        assert_not_found(&context, &error, 0);
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
