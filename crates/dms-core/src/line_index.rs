//! Converts byte offsets into 1-based `line:column` positions.
//!
//! Columns are measured in **UTF-16 code units** so they line up with the
//! Monaco editor (and the LSP/VS Code convention) on the frontend.

use crate::model::{Position, Range};

pub struct LineIndex<'a> {
    src: &'a str,
    /// Byte offset of the first character on each line.
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut line_starts = vec![0usize];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        LineIndex { src, line_starts }
    }

    pub fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.src.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let line_start = self.line_starts[line];
        // Guard against landing in the middle of a multi-byte sequence.
        let slice_end = floor_char_boundary(self.src, offset);
        let col = self.src[line_start..slice_end].encode_utf16().count();
        Position {
            line: (line + 1) as u32,
            col: (col + 1) as u32,
        }
    }

    pub fn range(&self, span: (usize, usize)) -> Range {
        let start = self.position(span.0);
        let end = self.position(span.1);
        Range {
            start_line: start.line,
            start_col: start.col,
            end_line: end.line,
            end_col: end.col,
        }
    }
}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_line_is_one() {
        let idx = LineIndex::new("abc");
        let p = idx.position(0);
        assert_eq!((p.line, p.col), (1, 1));
        let p = idx.position(2);
        assert_eq!((p.line, p.col), (1, 3));
    }

    #[test]
    fn newlines_advance_line() {
        let idx = LineIndex::new("a\nbb\nccc");
        let p = idx.position(2); // 'b'
        assert_eq!((p.line, p.col), (2, 1));
        let p = idx.position(5); // 'c'
        assert_eq!((p.line, p.col), (3, 1));
    }

    #[test]
    fn utf16_columns() {
        // "é" is 2 bytes in UTF-8 but 1 UTF-16 unit.
        let idx = LineIndex::new("éx");
        let p = idx.position(2); // byte offset of 'x'
        assert_eq!((p.line, p.col), (1, 2));
    }
}
