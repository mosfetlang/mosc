/// The result of every parser method.
pub type ParserResult<T> = Result<T, ParserResultError>;

/// The type of errors that parser method can return.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ParserResultError {
    NotFound,
    Error,
}
