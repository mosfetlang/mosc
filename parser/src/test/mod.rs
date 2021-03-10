use doclog::blocks::LogBlock;
use doclog::Log;

use crate::constants::LOG_ERROR_ID_TITLE;
use crate::constants::LOG_WARNING_ID_TITLE;
use crate::context::ParserContext;
use crate::parsers::ParserResultError;
use crate::ParserError;
use crate::ParserWarning;

pub fn assert_warning(context: &ParserContext, warning_type: ParserWarning) {
    let messages = context.messages();

    assert_eq!(messages.len(), 1, "The messages length is incorrect");

    assert_warning_message(&messages[0], warning_type);
}

pub fn assert_warning_message(message: &Log, warning_type: ParserWarning) {
    let indent_block = message.blocks().last().unwrap();
    let wid_block = match indent_block {
        LogBlock::Indent(v) => {
            let log = v.get_log();
            log.blocks().last().unwrap()
        }
        _ => panic!("The wid must be the last block of the last indent"),
    };

    match wid_block {
        LogBlock::Note(v) => {
            assert_eq!(
                v.get_title().as_str(),
                LOG_WARNING_ID_TITLE.as_str(),
                "The eid must be the last block"
            );

            assert_eq!(
                v.get_message().as_str(),
                format!("{:?}", warning_type).as_str(),
                "The error type is incorrect"
            );
        }
        _ => panic!("The eid must be the last block"),
    }
}

pub fn assert_error(context: &ParserContext, error: &ParserResultError, error_type: ParserError) {
    assert_eq!(error, &ParserResultError::Error, "The error is incorrect");

    let messages = context.messages();

    assert_eq!(messages.len(), 1, "The messages length is incorrect");

    let indent_block = messages[0].blocks().last().unwrap();
    let eid_block = match indent_block {
        LogBlock::Indent(v) => {
            let log = v.get_log();
            log.blocks().last().unwrap()
        }
        _ => panic!("The eid must be the last block of the last indent"),
    };

    match eid_block {
        LogBlock::Note(v) => {
            assert_eq!(
                v.get_title().as_str(),
                LOG_ERROR_ID_TITLE.as_str(),
                "The eid must be the last block"
            );

            assert_eq!(
                v.get_message().as_str(),
                format!("{:?}", error_type).as_str(),
                "The error type is incorrect"
            );
        }
        _ => panic!("The eid must be the last block"),
    }
}
