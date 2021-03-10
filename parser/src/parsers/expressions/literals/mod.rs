pub use numbers::*;

use crate::context::ParserContext;
use crate::io::{Reader, Span};
use crate::parsers::{ParserResult, ParserResultError};
use crate::ParserNode;

pub mod integer;
mod numbers;

/// A literal value in the Mosfet language, like a number, string, etc.
#[derive(Debug)]
pub enum Literal {
    Number(Number),
}

impl Literal {
    // GETTERS ----------------------------------------------------------------

    /// The span of the node.
    pub fn span(&self) -> &Span {
        match self {
            Literal::Number(n) => n.span(),
        }
    }

    // STATIC METHODS ---------------------------------------------------------

    /// Parses a literal.
    pub fn parse(reader: &mut Reader, context: &mut ParserContext) -> ParserResult<Literal> {
        match Number::parse(reader, context) {
            Ok(node) => return Ok(Literal::Number(node)),
            Err(ParserResultError::NotFound) => { /* Ignore because not found */ }
            Err(ParserResultError::Error) => return Err(ParserResultError::Error),
        }

        Err(ParserResultError::NotFound)
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
    // fn test_parse_number() {
    //     // Decimal without prefix.
    //     let mut reader = Reader::from_str("25/rest");
    //     let literal = Literal::parse(&mut reader, &ParserContext::default())
    //         .expect("The parser must succeed");
    //
    //     if let Literal::Number(number) = literal {
    //         assert_eq!(number.span().content(), "25", "The span is incorrect");
    //         assert_eq!(
    //             number.number(),
    //             &BigRational::from_integer(ToBigInt::to_bigint(&25).unwrap()),
    //             "The number is incorrect"
    //         );
    //     }
    //     // FIXME(juliotpaez): uncomment when there are more literals.
    //     // else {
    //     //     panic!("The literal is incorrect");
    //     // }
    // }

    #[test]
    fn test_parse_err_not_found() {
        let mut reader = Reader::from_str("-");
        let mut context = ParserContext::default();
        let error =
            Literal::parse(&mut reader, &mut context).expect_err("The parser must not succeed");

        assert_eq!(error, ParserResultError::NotFound, "The error is incorrect");
        assert_eq!(reader.offset(), 0, "The offset is incorrect");
    }
}
