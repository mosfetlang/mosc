use crate::errors::ParserError;
use crate::io::{Reader, Span};
use crate::parsers::commons::identifier::Identifier;
use crate::parsers::expressions::literals::Literal;
use crate::parsers::{ParserContext, ParserResult};

pub mod literals;

/// A expression in the Mosfet language, like a value or variable access.
#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    VariableAccess(Identifier),
}

impl Expression {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        match self {
            Expression::Literal(n) => n.span(),
            Expression::VariableAccess(n) => n.span(),
        }
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses an expression.
    pub fn parse(reader: &mut Reader, context: &ParserContext) -> ParserResult<Expression> {
        match Literal::parse(reader, context) {
            Ok(node) => return Ok(Expression::Literal(node)),
            Err(ParserError::NotFound) => { /* Ignore */ }
            Err(e) => return Err(e),
        }

        match Identifier::parse(reader, context) {
            Ok(node) => return Ok(Expression::VariableAccess(node)),
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
    use super::*;

    // TODO
    // #[test]
    // fn test_parse_literal() {
    //     let mut reader = Reader::from_str("25/rest");
    //     let expression = Expression::parse(&mut reader, &ParserContext::default())
    //         .expect("The parser must succeed");
    //
    //     if let Expression::Literal(literal) = expression {
    //         if let Literal::Number(number) = literal {
    //             assert_eq!(number.span().content(), "25", "The span is incorrect");
    //             assert_eq!(
    //                 number.number(),
    //                 &BigRational::from_integer(ToBigInt::to_bigint(&25).unwrap()),
    //                 "The number is incorrect"
    //             );
    //         }
    //         // FIXME(juliotpaez): uncomment when there are more literals.
    //         // else {
    //         //     panic!("The literal is incorrect");
    //         // }
    //     } else {
    //         panic!("The literal is incorrect");
    //     }
    // }

    #[test]
    fn test_parse_variable_access() {
        let mut reader = Reader::from_str("name/rest");
        let expression = Expression::parse(&mut reader, &ParserContext::default())
            .expect("The parser must succeed");

        if let Expression::VariableAccess(identifier) = expression {
            assert_eq!(identifier.name(), "name", "The name is incorrect");
        } else {
            panic!("The literal is incorrect");
        }
    }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let expression = Expression::parse(&mut reader, &ParserContext::default())
            .expect_err("The parser must not succeed");

        assert!(
            expression.variant_eq(&ParserError::NotFound),
            "The error is incorrect"
        );
        assert_eq!(reader.offset(), 0, "The offset is incorrect");
    }
}
