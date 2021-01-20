/// The error that will return all Parser methods and components in case something went wrong.
#[derive(Debug, Clone)]
pub enum ParserError {
    /// The parser cannot find a valid result with the input.
    NotFound,
}

impl ParserError {
    // METHODS ----------------------------------------------------------------

    /// Whether the two variants are the same or not.
    pub fn variant_eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
