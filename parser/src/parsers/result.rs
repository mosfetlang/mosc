use crate::errors::ParserError;

/// The result of every parser method.
pub type ParserResult<T> = Result<T, Option<ParserError>>;
