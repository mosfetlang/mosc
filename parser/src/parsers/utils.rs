use arcstr::ArcStr;

use doclog::blocks::DocumentBlock;
use doclog::Log;

use crate::constants::LOG_CODE_TITLE;
use crate::constants::LOG_ERROR_ID_TITLE;
use crate::constants::LOG_WARNING_ID_TITLE;
use crate::io::{Cursor, Reader};
use crate::parsers::{ParserResult, ParserResultError};
use crate::ParserError;
use crate::ParserWarning;

/// Helps to manage the initial cursor of a parser method and to restore a result cannot be found.
pub fn cursor_manager<F, T>(reader: &mut Reader, method: F) -> ParserResult<T>
where
    F: FnOnce(&mut Reader, &Cursor) -> ParserResult<T>,
{
    let init_cursor = reader.save_cursor();

    match method(reader, &init_cursor) {
        Ok(v) => Ok(v),
        Err(e) => {
            if e == ParserResultError::NotFound {
                reader.restore(init_cursor);
            }

            Err(e)
        }
    }
}

pub fn generate_warning_log<F, T: Into<ArcStr>>(
    warning_type: ParserWarning,
    title: T,
    builder: F,
) -> Log
where
    F: FnOnce(Log) -> Log,
{
    builder(Log::warn().title(title, true, false)).indent(2, |log| {
        log.note(LOG_WARNING_ID_TITLE.clone(), format!("{:?}", warning_type))
    })
}

pub fn generate_error_log<F, T: Into<ArcStr>>(error_type: ParserError, title: T, builder: F) -> Log
where
    F: FnOnce(Log) -> Log,
{
    builder(Log::error().title(title, true, false)).indent(2, |log| {
        log.note(LOG_ERROR_ID_TITLE.clone(), format!("{:?}", error_type))
    })
}

pub fn generate_source_code<F>(log: Log, reader: &Reader, builder: F) -> Log
where
    F: FnOnce(DocumentBlock) -> DocumentBlock,
{
    log.indent(2, |log| {
        log.document(reader.content().clone(), |doc| {
            let doc = doc.title(LOG_CODE_TITLE.clone());
            let doc = if let Some(file_path) = reader.file_path() {
                doc.file_path(file_path.clone())
            } else {
                doc
            };

            builder(doc)
        })
    })
}
