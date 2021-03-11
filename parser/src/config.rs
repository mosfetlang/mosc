/// The configuration related to ignoring some types of warnings.
#[derive(Debug)]
pub struct ParserIgnoreConfig {
    pub number_leading_zeroes: bool,
    pub number_trailing_zeroes: bool,
}

impl ParserIgnoreConfig {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `ParserIgnoreConfig` with the default configuration.
    pub fn new() -> ParserIgnoreConfig {
        ParserIgnoreConfig {
            number_leading_zeroes: false,
            number_trailing_zeroes: false,
        }
    }
}

impl Default for ParserIgnoreConfig {
    fn default() -> Self {
        Self::new()
    }
}
