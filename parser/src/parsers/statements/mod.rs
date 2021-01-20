pub use return_statement::*;
pub use variable_declaration::*;

use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::{ParserContext, ParserResult};

mod return_statement;
mod variable_declaration;

/// A statement in the Mosfet language, like a variable declaration.
#[derive(Debug)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    ReturnStatement(ReturnStatement),
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
    pub fn parse(reader: &mut Reader, context: &ParserContext) -> ParserResult<Statement> {
        match VariableDeclaration::parse(reader, context) {
            Ok(node) => return Ok(Statement::VariableDeclaration(node)),
            Err(ParserError::NotFound) => { /* Ignore */ }
            Err(e) => return Err(e),
        }

        match ReturnStatement::parse(reader, context) {
            Ok(node) => return Ok(Statement::ReturnStatement(node)),
            Err(ParserError::NotFound) => { /* Ignore */ }
            Err(e) => return Err(e),
        }

        Err(ParserError::NotFound)
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::parsers::expressions::Expression;

    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let mut reader = Reader::from_str("let test = a");
        let statement = Statement::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

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
        let statement = Statement::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

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
        let statement = Statement::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            statement.variant_eq(&ParserError::NotFound),
            "The error is incorrect"
        );
        assert_eq!(reader.offset(), 0, "The offset is incorrect");
    }
}
