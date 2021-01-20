use crate::errors::ParserError;
use crate::io::{Cursor, Reader};
use crate::parsers::ParserResult;

/// Helps to manage the initial cursor of a parser method and to restore a result cannot be found.
pub fn cursor_manager<F, T>(reader: &mut Reader, method: F) -> ParserResult<T>
where
    F: FnOnce(&mut Reader, &Cursor) -> ParserResult<T>,
{
    let init_cursor = reader.save();
    let result = method(reader, &init_cursor);

    if let Err(ParserError::NotFound) = result {
        reader.restore(init_cursor);
    }

    result
}
