use std::sync::Arc;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::expressions::literals::Literal;
use crate::parsers::{ParserResult, ParserResultError};
use crate::ParserNode;

pub mod literals;

/// A expression in the Mosfet language, like a value or variable access.
#[derive(Debug)]
pub enum Expression {
    Literal(Arc<Literal>),
    VariableAccess(Arc<Identifier>),
}

impl Expression {
    // STATIC METHODS ---------------------------------------------------------

    /// Parses an expression.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<Expression> {
        match Literal::parse(reader, context) {
            Ok(node) => return Ok(Expression::Literal(Arc::new(node))),
            Err(ParserResultError::NotFound) => { /* Ignore because not found */ }
            Err(ParserResultError::Error) => return Err(ParserResultError::Error),
        }

        match Identifier::parse(reader, context) {
            Ok(node) => return Ok(Expression::VariableAccess(Arc::new(node))),
            Err(ParserResultError::NotFound) => { /* Ignore because not found */ }
            Err(ParserResultError::Error) => return Err(ParserResultError::Error),
        }

        Err(ParserResultError::NotFound)
    }
}

impl ParserNode for Expression {
    fn span(&self) -> &Arc<Span> {
        match self {
            Expression::Literal(n) => n.span(),
            Expression::VariableAccess(n) => n.span(),
        }
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::test::assert_not_found;
    use crate::ParserNode;

    use super::*;

    #[test]
    fn test_parse_literal() {
        let mut reader = Reader::from_str("25/rest");
        let mut context = ParserContext::default();
        let expression =
            Expression::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Expression::Literal(literal) = expression {
            if let Literal::Number(number) = literal.as_ref() {
                assert_eq!(number.span().content(), "25", "The span is incorrect");
            }
        // FIXME(juliotpaez): uncomment when there are more literals.
        // else {
        //     panic!("The literal type is incorrect");
        // }
        } else {
            panic!("The expression type is incorrect");
        }
    }

    #[test]
    fn test_parse_variable_access() {
        let mut reader = Reader::from_str("name/rest");
        let mut context = ParserContext::default();
        let expression =
            Expression::parse(&mut reader, &mut context).expect("The parser must succeed");

        if let Expression::VariableAccess(identifier) = expression {
            assert_eq!(identifier.name(), "name", "The name is incorrect");
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let mut context = ParserContext::default();
        let error =
            Expression::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_not_found(&context, &error, 0);
    }
}
