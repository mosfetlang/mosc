use std::sync::Arc;

pub use return_statement::*;
pub use variable_declaration::*;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::{ParserResult, ParserResultError};
use crate::ParserNode;

mod return_statement;
mod variable_declaration;

/// A statement in the Mosfet language, like a variable declaration.
#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(Arc<VariableDeclaration>),
    ReturnStatement(Arc<ReturnStatement>),
}

impl Statement {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        match self {
            Statement::VariableDeclaration(n) => n.span(),
            Statement::ReturnStatement(n) => n.span(),
        }
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a statement.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<Statement> {
        match VariableDeclaration::parse(reader, context) {
            Ok(node) => return Ok(Statement::VariableDeclaration(Arc::new(node))),
            Err(ParserResultError::NotFound) => { /* Ignore because not found */ }
            Err(ParserResultError::Error) => return Err(ParserResultError::Error),
        }

        match ReturnStatement::parse(reader, context) {
            Ok(node) => return Ok(Statement::ReturnStatement(Arc::new(node))),
            Err(ParserResultError::NotFound) => { /* Ignore because not found */ }
            Err(ParserResultError::Error) => return Err(ParserResultError::Error),
        }

        Err(ParserResultError::NotFound)
    }
}

impl ParserNode for Statement {
    fn span(&self) -> &Arc<Span> {
        match self {
            Statement::VariableDeclaration(n) => n.span(),
            Statement::ReturnStatement(n) => n.span(),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::parsers::expressions::Expression;
    use crate::test::assert_not_found;

    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let mut reader = Reader::from_str("let test = a");
        let mut context = ParserContext::default();
        let statement =
            Statement::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Statement::VariableDeclaration(declaration) = statement {
            assert_eq!(declaration.name().name(), "test", "The name is incorrect");
            if let Expression::VariableAccess(identifier) = declaration.expression() {
                assert_eq!(identifier.name(), "a", "The literal access is incorrect");
            } else {
                panic!("The literal is incorrect");
            }
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_variable_access() {
        let mut reader = Reader::from_str("return test");
        let mut context = ParserContext::default();
        let statement =
            Statement::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Statement::ReturnStatement(statement) = statement {
            if let Expression::VariableAccess(identifier) = statement.expression() {
                assert_eq!(identifier.name(), "test", "The literal access is incorrect");
            } else {
                panic!("The literal is incorrect");
            }
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let mut context = ParserContext::default();
        let error =
            Statement::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_not_found(&context, &error, 0);
    }
}
