use crate::io::Reader;

/// A specific position inside a `Reader`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cursor {
    reader_id: usize,
    offset: usize,
    char_offset: usize,
    line: usize,
    column: usize,
}

impl Cursor {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `Cursor` with the specified data.
    pub(in crate::io::readers) fn new(
        reader_id: usize,
        offset: usize,
        char_offset: usize,
        line: usize,
        column: usize,
    ) -> Cursor {
        Cursor {
            reader_id,
            offset,
            char_offset,
            line,
            column,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The id of the `Reader` this cursor belongs to.
    pub(in crate::io::readers) fn reader_id(&self) -> usize {
        self.reader_id
    }

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

    // SETTERS ----------------------------------------------------------------

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

    // METHODS ----------------------------------------------------------------

    /// Returns whether this cursor belong to the `reader` or not.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let reader1 = Reader::from_str("test1");
    /// let reader2 = Reader::from_str("test2");
    /// let cursor1 = reader1.save_cursor();
    /// let cursor2 = reader2.save_cursor();
    ///
    /// assert_eq!(cursor1.belongs_to(&reader1), true);
    /// assert_eq!(cursor2.belongs_to(&reader1), false);
    /// ```
    pub fn belongs_to(&self, reader: &Reader) -> bool {
        self.reader_id == reader.id
    }
}
