use std::sync::Arc;

use memchr::{memchr, memrchr};

use crate::io::Cursor;

/// A Span is a set of meta information about the location of a substring.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Span {
    content: Arc<String>,
    start_cursor: Arc<Cursor>,
    end_cursor: Arc<Cursor>,
}

impl Span {
    // CONSTRUCTORS -----------------------------------------------------------

    /// Builds a new `Span` with the specified data.
    pub(in crate) fn new(
        content: Arc<String>,
        start_cursor: Arc<Cursor>,
        end_cursor: Arc<Cursor>,
    ) -> Span {
        Span {
            content,
            start_cursor,
            end_cursor,
        }
    }

    // GETTERS ----------------------------------------------------------------

    /// The whole content the `Span` belongs to.
    pub fn whole_content(&self) -> &Arc<String> {
        &self.content
    }

    /// The content of the `Span`.
    pub fn content(&self) -> &str {
        &self.content[self.start_cursor.offset()..self.end_cursor.offset()]
    }

    /// The content before the `Span`.
    pub fn content_before(&self) -> &str {
        &self.content[..self.start_cursor.offset()]
    }

    /// The content after the `Span`.
    pub fn content_after(&self) -> &str {
        &self.content[self.end_cursor.offset()..]
    }

    /// The start position of the `Span` in bytes.
    pub fn start_cursor(&self) -> &Cursor {
        &self.start_cursor
    }

    /// The end position of the `Span` in bytes.
    pub fn end_cursor(&self) -> &Cursor {
        &self.end_cursor
    }

    /// The length of the `Span` in bytes.
    pub fn len(&self) -> usize {
        self.end_cursor.offset() - self.start_cursor.offset()
    }

    /// The length of the `Span` in characters.
    pub fn char_length(&self) -> usize {
        self.end_cursor.char_offset() - self.start_cursor.char_offset()
    }

    /// Returns the line(s) in which the `Span` is contained.
    /// If it is composed of more than one line, the result will be all the lines.
    ///
    /// # Example
    ///
    /// ```
    /// # use parser::io::Reader;
    /// let mut reader = Reader::from_str("This\nis\nthe\nfragment");
    ///
    /// // ... prepare the span to contain: "his\nis\nt" ...
    /// # reader.read("T");
    /// # let from_cursor = reader.save_cursor();
    /// # reader.read("his\nis\nt");
    /// # let to_cursor = reader.save_cursor();
    /// let span = reader.substring(&from_cursor, &to_cursor);
    ///
    /// // Get its lines.
    /// assert_eq!(span.lines(), "This\nis\nthe");
    /// ```
    pub fn lines(&self) -> &str {
        let start_index = match memrchr(b'\n', self.content_before().as_bytes()) {
            Some(v) => v + 1,
            None => 0,
        };

        let end_index = match memchr(b'\n', self.content_after().as_bytes()) {
            Some(v) => v + self.end_cursor.offset(),
            None => self.content.len(),
        };

        &self.content[start_index..end_index]
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lines_single_line() {
        let text = "This\nis\nthe\ntest";
        let span = Span::new(
            Arc::new(text.to_string()),
            Arc::new(Cursor::new(0, 1, 0, 0, 0)), // Only offset matters.
            Arc::new(Cursor::new(0, 1, 0, 0, 0)), // Only offset matters.
        );

        assert_eq!(span.lines(), "This", "The lines is incorrect");

        // Check at \n
        let text = "This\nis\nthe\ntest";
        let span = Span::new(
            Arc::new(text.to_string()),
            Arc::new(Cursor::new(0, 4, 0, 0, 0)), // Only offset matters.
            Arc::new(Cursor::new(0, 4, 0, 0, 0)), // Only offset matters.
        );

        assert_eq!(span.lines(), "This", "The lines is incorrect");

        // Check next of \n
        let text = "This\nis\nthe\ntest";
        let span = Span::new(
            Arc::new(text.to_string()),
            Arc::new(Cursor::new(0, 5, 0, 0, 0)), // Only offset matters.
            Arc::new(Cursor::new(0, 5, 0, 0, 0)), // Only offset matters.
        );

        assert_eq!(span.lines(), "is", "The lines is incorrect");
    }

    #[test]
    fn test_lines_multiline() {
        let text = "This\nis\nthe\ntest";
        let span = Span::new(
            Arc::new(text.to_string()),
            Arc::new(Cursor::new(0, 5, 0, 0, 0)), // Only offset matters.
            Arc::new(Cursor::new(0, 8, 0, 0, 0)), // Only offset matters.
        );

        assert_eq!(span.lines(), "is\nthe", "The lines is incorrect");
    }
}
