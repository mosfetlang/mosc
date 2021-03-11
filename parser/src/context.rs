use doclog::Log;

use crate::ParserIgnoreConfig;

/// The context of the parser that contains all contextual information of the parsing.
#[derive(Debug)]
pub struct ParserContext {
    messages: Vec<Log>,
    ignore: ParserIgnoreConfig,
}

impl ParserContext {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `ParserContext` with the default configuration.
    pub fn new(ignore: ParserIgnoreConfig) -> ParserContext {
        ParserContext {
            messages: Vec::new(),
            ignore,
        }
    }

    // GETTERS ----------------------------------------------------------------

    pub fn messages(&self) -> &Vec<Log> {
        &self.messages
    }

    pub fn ignore(&self) -> &ParserIgnoreConfig {
        &self.ignore
    }

    // METHODS ----------------------------------------------------------------

    pub fn add_message(&mut self, log: Log) {
        self.messages.push(log);
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new(ParserIgnoreConfig::default())
    }
}
