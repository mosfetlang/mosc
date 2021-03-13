use std::sync::Arc;

use doclog::Color;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::commons::whitespaces::Whitespace;
use crate::parsers::expressions::Expression;
use crate::parsers::result::ParserResult;
use crate::parsers::utils::{cursor_manager, generate_error_log, generate_source_code};
use crate::parsers::ParserResultError;
use crate::{ParserError, ParserNode};

static KEYWORD: &str = "let";
static ASSIGN_OPERATOR: &str = "=";

/// A variable declaration with a compulsory expression.
#[derive(Debug)]
pub struct VariableDeclaration {
    span: Arc<Span>,
    name: Arc<Identifier>,
    expression: Arc<Expression>,
    pre_name_whitespace: Arc<Whitespace>,
    pre_assign_operator_whitespace: Arc<Whitespace>,
    pre_expression_whitespace: Arc<Whitespace>,
}

impl VariableDeclaration {
    // GETTERS ----------------------------------------------------------------

    pub fn name(&self) -> &Arc<Identifier> {
        &self.name
    }

    pub fn expression(&self) -> &Arc<Expression> {
        &self.expression
    }

    pub fn pre_name_whitespace(&self) -> &Arc<Whitespace> {
        &self.pre_name_whitespace
    }

    pub fn pre_assign_operator_whitespace(&self) -> &Arc<Whitespace> {
        &self.pre_assign_operator_whitespace
    }

    pub fn pre_expression_whitespace(&self) -> &Arc<Whitespace> {
        &self.pre_expression_whitespace
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

            let pre_name_whitespace = Whitespace::parse_multiline_or_default(reader, context);

            let name = match Identifier::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    context.add_message(generate_error_log(
                        ParserError::MissingNameInVariableDeclaration,
                        arcstr::literal!("The variable name is missing"),
                        |log| {
                            generate_source_code(log, &reader, |doc| {
                                doc.highlight_section(
                                    init_cursor.byte_offset()
                                        ..pre_name_whitespace.span().start_cursor().byte_offset(),
                                    None,
                                    Some(Color::Magenta),
                                )
                                .highlight_cursor(
                                    pre_name_whitespace.span().start_cursor().byte_offset(),
                                    Some(arcstr::literal!("Insert an identifier here")),
                                    None,
                                )
                            })
                        },
                    ));

                    return Err(ParserResultError::Error);
                }
            };

            let pre_assign_operator_whitespace =
                Whitespace::parse_multiline_or_default(reader, context);

            if !reader.read(ASSIGN_OPERATOR) {
                context.add_message(generate_error_log(
                    ParserError::MissingAssignOperatorInVariableDeclaration,
                    arcstr::literal!("The assign operator is required after the variable name to define its value"),
                    |log| {
                        generate_source_code(log, &reader, |doc| {
                            doc.highlight_section(
                                init_cursor.byte_offset()
                                    ..pre_assign_operator_whitespace
                                    .span()
                                    .start_cursor()
                                    .byte_offset(),
                                None,
                                Some(Color::Magenta),
                            )
                                .highlight_cursor(
                                    pre_assign_operator_whitespace
                                        .span()
                                        .start_cursor()
                                        .byte_offset(),
                                    Some(
                                        format!(
                                            "Insert the assign operator '{}' here",
                                            ASSIGN_OPERATOR
                                        )
                                            .into(),
                                    ),
                                    None,
                                )
                        })
                    },
                ));

                return Err(ParserResultError::Error);
            }

            let pre_expression_whitespace = Whitespace::parse_multiline_or_default(reader, context);

            let expression = match Expression::parse(reader, context) {
                Ok(v) => v,
                Err(_) => {
                    context.add_message(generate_error_log(
                        ParserError::MissingExpressionInVariableDeclaration,
                        arcstr::literal!("An expression is expected after the assign operator"),
                        |log| {
                            generate_source_code(log, &reader, |doc| {
                                doc.highlight_section(
                                    init_cursor.byte_offset()
                                        ..pre_expression_whitespace
                                            .span()
                                            .start_cursor()
                                            .byte_offset(),
                                    None,
                                    Some(Color::Magenta),
                                )
                                .highlight_cursor(
                                    pre_expression_whitespace
                                        .span()
                                        .start_cursor()
                                        .byte_offset(),
                                    Some(arcstr::literal!("Insert an expression here")),
                                    None,
                                )
                            })
                        },
                    ));

                    return Err(ParserResultError::Error);
                }
            };

            let span = Arc::new(reader.substring_to_current(&init_cursor));
            Ok(VariableDeclaration {
                span,
                name: Arc::new(name),
                expression: Arc::new(expression),
                pre_name_whitespace: Arc::new(pre_name_whitespace),
                pre_expression_whitespace: Arc::new(pre_expression_whitespace),
                pre_assign_operator_whitespace: Arc::new(pre_assign_operator_whitespace),
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
        let mut reader = Reader::from_content(arcstr::literal!("let    test   =   a"));
        let mut context = ParserContext::default();
        let declaration =
            VariableDeclaration::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(declaration.name.content(), "test", "The name is incorrect");
        if let Expression::VariableAccess(identifier) = declaration.expression.as_ref() {
            assert_eq!(identifier.content(), "a", "The literal access is incorrect");
        } else {
            panic!("The literal is incorrect");
        }

        // Without whitespaces.
        let mut reader = Reader::from_content(arcstr::literal!("let test=a"));
        let mut context = ParserContext::default();
        let declaration =
            VariableDeclaration::parse(&mut reader, &mut context).expect("The parser must succeed");

        assert_eq!(declaration.name.content(), "test", "The name is incorrect");
        if let Expression::VariableAccess(identifier) = declaration.expression.as_ref() {
            assert_eq!(identifier.content(), "a", "The literal access is incorrect");
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_content(arcstr::literal!("-"));
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_not_found(&context, &error, 0);
    }

    #[test]
    fn test_parse_err_missing_variable_name() {
        let mut reader = Reader::from_content(arcstr::literal!("let"));
        let mut context = ParserContext::default();
        let error = VariableDeclaration::parse(&mut reader, &mut context)
            .expect_err("The parser must not succeed");

        assert_error(
            &context,
            &error,
            ParserError::MissingNameInVariableDeclaration,
        );
    }

    #[test]
    fn test_parse_err_missing_assign_operator() {
        let mut reader = Reader::from_content(arcstr::literal!("let test"));
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
        let mut reader = Reader::from_content(arcstr::literal!("let test ="));
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
