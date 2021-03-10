use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::expressions::Expression;
use crate::parsers::result::ParserResult;
use crate::parsers::utils::cursor_manager;
use crate::parsers::ParserResultError;
use crate::ParserNode;

static KEYWORD: &str = "let";
static ASSIGN_OPERATOR: &str = "=";

/// A variable declaration with a compulsory expression.
#[derive(Debug)]
pub struct VariableDeclaration {
    span: Arc<Span>,
    name: Identifier,
    expression: Expression,
}

impl VariableDeclaration {
    // GETTERS ----------------------------------------------------------------

    /// The name of the variable declaration.
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// The expression of the variable declaration.
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a variable declaration.
    pub fn parse(
        reader: &mut Reader,
        context: &mut ParserContext,
    ) -> ParserResult<VariableDeclaration> {
        cursor_manager(reader, |reader, init_cursor| {
            if !Identifier::parse_keyword(reader, context, KEYWORD) {
                return Err(ParserResultError::NotFound);
            }

            let whitespace = Whitespace::parse_inline(reader, context);

            let name = match Identifier::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    // TODO improve with a log.
                    return Err(ParserResultError::Error);
                    // return Err(ParserError::MissingVariableNameInVariableDeclaration(
                    //     whitespace
                    //         .map(|node| node.span().start_cursor().clone())
                    //         .unwrap_or(reader.save()),
                    // ));
                }
            };

            let whitespace = Whitespace::parse_inline(reader, context);

            if !reader.read(ASSIGN_OPERATOR) {
                // TODO improve with a log.
                return Err(ParserResultError::Error);
                // return Err(ParserError::MissingAssignOperatorInVariableDeclaration(
                //     whitespace
                //         .map(|node| node.span().start_cursor().clone())
                //         .unwrap_or(reader.save()),
                // ));
            }

            let whitespace = Whitespace::parse_inline(reader, context);

            let expression = match Expression::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    // TODO improve with a log.
                    return Err(ParserResultError::Error);
                    // return Err(ParserError::MissingExpressionInVariableDeclaration(
                    //     whitespace
                    //         .map(|node| node.span().start_cursor().clone())
                    //         .unwrap_or(reader.save()),
                    // ));
                }
            };

            let span = Arc::new(reader.substring_to_current(&init_cursor));
            Ok(VariableDeclaration {
                span,
                name,
                expression,
            })
        })
    }
}

impl ParserNode for VariableDeclaration {
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
        let mut reader = Reader::from_str("let    test   =   a");
        let mut context = ParserContext::default();
        let declaration =
            VariableDeclaration::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(declaration.name.name(), "test", "The name is incorrect");
        if let Expression::VariableAccess(identifier) = declaration.expression {
            assert_eq!(identifier.name(), "a", "The literal access is incorrect");
        } else {
            panic!("The literal is incorrect");
        }

        // Without whitespaces.
        let mut reader = Reader::from_str("let test=a");
        let mut context = ParserContext::default();
        let declaration =
            VariableDeclaration::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(declaration.name.name(), "test", "The name is incorrect");
        if let Expression::VariableAccess(identifier) = declaration.expression {
            assert_eq!(identifier.name(), "a", "The literal access is incorrect");
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_not_found(&context, &error, 0);
    }

    #[test]
    fn test_parse_err_missing_variable_name() {
        let mut reader = Reader::from_str("let");
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MissingVariableNameInVariableDeclaration,
        );
    }

    #[test]
    fn test_parse_err_missing_assign_operator() {
        let mut reader = Reader::from_str("let test");
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MissingAssignOperatorInVariableDeclaration,
        );
    }

    #[test]
    fn test_parse_err_missing_expression() {
        let mut reader = Reader::from_str("let test =");
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MissingExpressionInVariableDeclaration,
        );
    }
}
