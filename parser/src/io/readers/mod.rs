use std::ops::RangeInclusive;
use std::sync::Arc;

use bytecount::num_chars;
use memchr::Memchr;

pub use cursor::*;
pub use span::*;

mod cursor;
mod span;

/// A `String` reader that moves a cursor the reader updated.
#[derive(Debug)]
pub struct Reader {
    file_path: Option<Arc<String>>,
    content: Arc<String>,
    cursor: Cursor,
}

impl Reader {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Create a new `Reader` with the specified `file_path` and `content`.
    pub fn new(file_path: Option<Arc<String>>, content: Arc<String>) -> Reader {
        Reader {
            file_path,
            content,
            cursor: Cursor::new(0, 0, 1, 1),
        }
    }

    /// Create a new `Reader` with the specified `content`.
    pub fn from_str(content: &str) -> Reader {
        Self::new(None, Arc::new(content.to_string()))
    }

    /// Create a new `Reader` with the specified `content`.
    pub fn from_content(content: Arc<String>) -> Reader {
        Self::new(None, content)
    }

    // GETTERS ----------------------------------------------------------------

    /// The file path of the `Reader` if there's any.
    pub fn file_path(&self) -> &Option<Arc<String>> {
        &self.file_path
    }

    /// The content of the `Reader`.
    pub fn content(&self) -> &Arc<String> {
        &self.content
    }

    /// The position of the `Reader` in bytes.
    pub fn offset(&self) -> usize {
        self.cursor.offset()
    }

    /// The position of the `Cursor` in characters.
    /// It starts at char 0.
    pub fn char_offset(&self) -> usize {
        self.cursor.char_offset()
    }

    /// The line number of the current position.
    /// It starts at line 1.
    pub fn line(&self) -> usize {
        self.cursor.line()
    }

    /// The column number of the current position.
    /// It starts at column 1.
    pub fn column(&self) -> usize {
        self.cursor.column()
    }

    /// The remaining content as an `Slice`.
    pub fn remaining_content(&self) -> &str {
        &self.content.as_str()[self.cursor.offset()..]
    }

    /// The remaining content as an `Span`.
    pub fn remaining_content_span(&self) -> Span {
        let mut aux_reader = Reader::from_content(self.content.clone());
        aux_reader.consume(self.content.len());

        Span::new(
            self.content.clone(),
            self.cursor.clone(),
            aux_reader.cursor.clone(),
        )
    }

    /// The length in bytes of the content that is not already read.
    pub fn remaining_length(&self) -> usize {
        self.content.len() - self.offset()
    }

    /// The length in characters of the content that is not already read.
    pub fn remaining_char_length(&self) -> usize {
        num_chars(self.remaining_content().as_bytes())
    }

    // METHODS ----------------------------------------------------------------

    /// Consumes a `text` if present moving the start index forward.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("test");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.read("tes");
    /// assert!(result);
    /// assert_eq!(reader.offset(), 3);
    ///
    /// let result = reader.read("tes");
    /// assert!(!result);
    /// assert_eq!(reader.offset(), 3);
    /// ```
    pub fn read(&mut self, text: &str) -> bool {
        if self.continues_with(text) {
            self.consume(text.len());
            true
        } else {
            false
        }
    }

    /// Consumes one character if present in `interval` moving the start index forward.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("te");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.read_one_of(&['a'..='z']);
    /// assert_eq!(result, Some('t'));
    /// assert_eq!(reader.offset(), 1);
    ///
    /// let result = reader.read_one_of(&['a'..='z']);
    /// assert_eq!(result, Some('e'));
    /// assert_eq!(reader.offset(), 2);
    ///
    /// let result = reader.read_one_of(&['a'..='z']);
    /// assert_eq!(result, None);
    /// assert_eq!(reader.offset(), 2);
    /// ```
    pub fn read_one_of(&mut self, interval: &[RangeInclusive<char>]) -> Option<char> {
        if let Some(char) = self.continues_with_one_of(interval) {
            self.consume(char.len_utf8());
            Some(char)
        } else {
            None
        }
    }

    /// Consumes one or more characters if present in `interval` moving the start index forward.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.read_one_or_more_of(&['a'..='z']);
    /// assert_eq!(result, Some("this"));
    /// assert_eq!(reader.offset(), 4);
    ///
    /// let result = reader.read_one_or_more_of(&['a'..='z']);
    /// assert_eq!(result, None);
    /// assert_eq!(reader.offset(), 4);
    /// ```
    pub fn read_one_or_more_of(&mut self, interval: &[RangeInclusive<char>]) -> Option<&str> {
        if let Some(text) = self.continues_with_one_or_more_of(interval) {
            let length = text.len();
            self.consume(length);
            Some(&self.content.as_str()[self.offset() - length..self.offset()])
        } else {
            None
        }
    }

    /// Checks whether the reader continues with the specified `text`.
    /// This method does not consume the reader.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("test");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// assert_eq!(reader.continues_with("tes"), true);
    /// assert_eq!(reader.continues_with("this"), false);
    /// assert_eq!(reader.offset(), 0);
    /// ```
    pub fn continues_with(&self, text: &str) -> bool {
        let remaining = self.remaining_content();
        remaining.starts_with(text)
    }

    /// Checks whether the reader continues with one of the characters specified by `interval`.
    /// This method does not consume the reader.
    ///
    /// **Note**: this method requires `interval` be sorted.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("test");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.continues_with_one_of(&['a'..='z']);
    /// assert_eq!(result, Some('t'));
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.continues_with_one_of(&['A'..='Z']);
    /// assert_eq!(result, None);
    /// assert_eq!(reader.offset(), 0);
    /// ```
    pub fn continues_with_one_of(&self, interval: &[RangeInclusive<char>]) -> Option<char> {
        let remaining = self.remaining_content();
        let char = match remaining.chars().next() {
            Some(v) => v,
            None => return None,
        };

        if Self::check_inside(char, interval) {
            Some(char)
        } else {
            None
        }
    }

    /// Checks whether the reader continues with one or more of the characters specified by `interval`.
    /// This method does not consume the reader.
    ///
    /// **Note**: this method requires `interval` be sorted.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.continues_with_one_or_more_of(&['a'..='z']);
    /// assert_eq!(result, Some("this"));
    /// assert_eq!(reader.offset(), 0);
    ///
    /// let result = reader.continues_with_one_or_more_of(&['A'..='Z']);
    /// assert_eq!(result, None);
    /// assert_eq!(reader.offset(), 0);
    /// ```
    pub fn continues_with_one_or_more_of(&self, interval: &[RangeInclusive<char>]) -> Option<&str> {
        let remaining = self.remaining_content();

        let mut offset = 0;
        for char in remaining.chars() {
            if !Self::check_inside(char, interval) {
                break;
            }

            offset += char.len_utf8();
        }

        if offset == 0 {
            // No consumed characters.
            None
        } else {
            Some(&remaining[0..offset])
        }
    }

    /// Gets a `Span` that contains the susbstring delimited by both (`from`, `to`) cursors.
    /// The order of the cursors does not matter.
    ///
    /// # Safety
    ///
    /// This method will panic if any of both cursors do not belong to the current reader.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// reader.read("th");
    ///
    /// let from = reader.save();
    /// reader.read("is tes");
    ///
    /// let to = reader.save();
    ///
    /// assert_eq!(reader.substring(&from, &to).content(), "is tes");
    /// assert_eq!(reader.substring(&to, &from).content(), "is tes");
    /// ```
    pub fn substring(&self, from: &Cursor, to: &Cursor) -> Span {
        let (from, to) = if from.offset() <= to.offset() {
            (from, to)
        } else {
            (to, from)
        };

        Span::new(self.content.clone(), from.clone(), to.clone())
    }

    /// Gets a `Span` that contains the susbstring delimited by `from` and the current cursors.
    /// The order of the cursors does not matter.
    ///
    /// # Safety
    ///
    /// This method will panic if any of both cursors do not belong to the current reader.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// reader.read("th");
    ///
    /// let from = reader.save();
    /// reader.read("is tes");
    ///
    /// assert_eq!(reader.substring_to_current(&from).content(), "is tes");
    /// ```
    pub fn substring_to_current(&self, from: &Cursor) -> Span {
        let (from, to) = if from.offset() <= self.offset() {
            (from, &self.cursor)
        } else {
            (&self.cursor, from)
        };

        Span::new(self.content.clone(), from.clone(), to.clone())
    }

    /// Builds a new `Cursor` at the current position of the `Reader`.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// reader.read("th");
    ///
    /// let cursor = reader.save();
    ///
    /// assert_eq!(cursor.offset(), 2);
    /// ```
    pub fn save(&self) -> Cursor {
        self.cursor.clone()
    }

    /// Restores the reader to the specified `Cursor` state.
    ///
    /// # Safety
    ///
    /// This method is not checked so can create undefined behaviour if the cursor
    /// does not correspond to the reader.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("this test");
    /// let cursor = reader.save();
    ///
    /// assert_eq!(reader.offset(), 0);
    /// assert_eq!(cursor.offset(), 0);
    ///
    /// reader.read("th");
    /// let cursor2 = reader.save();
    ///
    /// assert_eq!(reader.offset(), 2);
    /// assert_eq!(cursor.offset(), 0);
    /// assert_eq!(cursor2.offset(), 2);
    ///
    /// reader.restore(cursor);
    ///
    /// assert_eq!(reader.offset(), 0);
    /// assert_eq!(cursor2.offset(), 2);
    /// ```
    pub fn restore(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    /// Consumes `count` bytes moving the start index forward.
    fn consume(&mut self, count: usize) {
        assert!(
            self.remaining_length() >= count,
            "count is greater than end position",
        );

        // Speed up method.
        if count == 0 {
            return;
        }

        let offset = self.offset();
        let new_offset = offset + count;
        let consumed_fragment = &self.content[offset..new_offset];
        let additional_chars = num_chars(consumed_fragment.as_bytes());
        let additional_lines = Memchr::new(b'\n', consumed_fragment.as_bytes()).count();

        // When the line change, count previous characters. Otherwise count only consumed chars to speed-up.
        let new_column = if additional_lines == 0 {
            self.column() + num_chars(consumed_fragment.as_bytes())
        } else {
            let bytes_before_self = &self.content[..new_offset];
            let start_position = match memchr::memrchr(b'\n', bytes_before_self.as_bytes()) {
                Some(pos) => new_offset - pos,
                None => new_offset + 1,
            };

            num_chars(bytes_before_self[new_offset - (start_position - 1)..].as_bytes()) + 1
        };

        self.cursor.set_offset(new_offset);
        self.cursor
            .set_char_offset(self.char_offset() + additional_chars);
        self.cursor.set_column(new_column);
        self.cursor.set_line(self.line() + additional_lines);
    }

    // STATIC -----------------------------------------------------------------

    /// Checks whether `char` is contained in `interval`.
    fn check_inside(char: char, interval: &[RangeInclusive<char>]) -> bool {
        for range in interval {
            // Exit early to optimize searching.
            if &char < range.start() {
                break;
            }

            if range.contains(&char) {
                return true;
            }
        }

        false
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_0() {
        let text = "This\nis\nthe\nfragment";
        let mut reader = Reader::from_str(text);
        reader.consume(0);

        assert_eq!(reader.offset(), 0, "The offset is incorrect");
        assert_eq!(reader.char_offset(), 0, "The char_offset is incorrect");
        assert_eq!(reader.line(), 1, "The line is incorrect");
        assert_eq!(reader.column(), 1, "The column is incorrect");
    }

    #[test]
    fn test_consume() {
        let text = "This\nis\nthe\nfragment";
        let mut reader = Reader::from_str(text);
        reader.consume(2);

        assert_eq!(reader.offset(), 2, "The offset is incorrect");
        assert_eq!(reader.char_offset(), 2, "The char_offset is incorrect");
        assert_eq!(reader.line(), 1, "The line is incorrect");
        assert_eq!(reader.column(), 3, "The column is incorrect");

        reader.consume(3);

        assert_eq!(reader.offset(), 5, "The offset is incorrect");
        assert_eq!(reader.char_offset(), 5, "The char_offset is incorrect");
        assert_eq!(reader.line(), 2, "The line is incorrect");
        assert_eq!(reader.column(), 1, "The column is incorrect");

        reader.consume(2);

        assert_eq!(reader.offset(), 7, "The offset is incorrect");
        assert_eq!(reader.char_offset(), 7, "The char_offset is incorrect");
        assert_eq!(reader.line(), 2, "The line is incorrect");
        assert_eq!(reader.column(), 3, "The column is incorrect");
    }

    #[test]
    fn test_consume_utf_chars() {
        let text = "モスフェト";
        let mut reader = Reader::from_str(text);
        reader.consume(3);

        assert_eq!(reader.offset(), 3, "The offset is incorrect");
        assert_eq!(reader.char_offset(), 1, "The char_offset is incorrect");
        assert_eq!(reader.line(), 1, "The line is incorrect");
        assert_eq!(reader.column(), 2, "The column is incorrect");
    }
}
