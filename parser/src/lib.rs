use std::sync::Arc;

pub use config::*;
pub use context::*;
pub use errors::*;
pub use warnings::*;

use crate::io::Span;

mod config;
mod constants;
mod context;
mod errors;
pub mod io;
pub mod parsers;
#[cfg(test)]
pub mod test;
mod warnings;

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
