/// The warnings that parsers can rise.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ParserWarning {
    NumberWithLeadingZeroes,
    NumberWithTrailingZeroes,
}
