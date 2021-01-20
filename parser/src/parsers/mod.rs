pub use result::*;

pub mod commons;
pub mod expressions;
mod result;

/// The context of the parser that contains all contextual information of the parsing.
#[derive(Debug)]
pub struct ParserContext {}

impl ParserContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `ParserContext` with the default configuration.
    pub fn new() -> ParserContext {
        ParserContext {}
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new()
    }
}
