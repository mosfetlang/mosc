/// A specific position inside a `Reader`.
#[derive(Debug, Clone)]
pub struct Cursor {
    offset: usize,
    char_offset: usize,
    line: usize,
    column: usize,
}

impl Cursor {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `Cursor` with the specified data.
    pub(in crate::io::readers) fn new(
        offset: usize,
        char_offset: usize,
        line: usize,
        column: usize,
    ) -> Cursor {
        Cursor {
            offset,
            char_offset,
            line,
            column,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The position of the `Cursor` in bytes.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The position of the `Cursor` in characters.
    pub fn char_offset(&self) -> usize {
        self.char_offset
    }

    /// The line number in which the `Cursor` is placed.
    /// It starts at line 1.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column number in which the `Cursor` is placed.
    /// It starts at column 1.
    pub fn column(&self) -> usize {
        self.column
    }

    // GETTERS ----------------------------------------------------------------

    pub(in crate::io::readers) fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub(in crate::io::readers) fn set_char_offset(&mut self, char_offset: usize) {
        self.char_offset = char_offset;
    }

    pub(in crate::io::readers) fn set_line(&mut self, line: usize) {
        self.line = line;
    }

    pub(in crate::io::readers) fn set_column(&mut self, column: usize) {
        self.column = column;
    }
}
