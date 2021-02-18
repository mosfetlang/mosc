use std::sync::Arc;

use crate::io::Span;

pub mod errors;
pub mod io;
pub mod parsers;

/// A trait that is implemented across all nodes belonging to the parser.
pub trait ParserNode {
    // GETTERS ----------------------------------------------------------------

    /// The `Span` that bounds the node.
    fn span(&self) -> &Arc<Span>;

    /// The whole content of the node.
    fn content(&self) -> &str {
        self.span().content()
    }
}
