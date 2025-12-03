//! Locator - efficient source location lookup.
//!
//! Inspired by Ruff's ruff_source_file crate.
//! Pre-computes line start offsets for O(log n) line lookups.

/// Index for fast byte offset to (line, column) conversions.
///
/// The index stores the byte offset of each line start, enabling
/// binary search for line number lookups.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offsets of line starts (0-indexed).
    /// The first element is always 0.
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Build a LineIndex from source bytes.
    ///
    /// Scans the source once to find all newline positions.
    pub fn from_source(source: &[u8]) -> Self {
        let mut line_starts = Vec::with_capacity(source.len() / 80); // Estimate ~80 chars per line
        line_starts.push(0);

        for (i, &byte) in source.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }

        Self { line_starts }
    }

    /// Get the line number (1-indexed) for a byte offset.
    ///
    /// Uses binary search for O(log n) performance.
    #[inline]
    pub fn line_number(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            // Offset is exactly at a line start
            Ok(line) => line + 1,
            // Offset is within a line
            Err(line) => line, // line is the insertion point, which equals line number (1-indexed)
        }
    }

    /// Get the column number (1-indexed) for a byte offset.
    #[inline]
    pub fn column_number(&self, offset: usize) -> usize {
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        let line_start = self.line_starts[line_index];
        offset - line_start + 1
    }

    /// Get both line and column (1-indexed) for a byte offset.
    #[inline]
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        let line_start = self.line_starts[line_index];
        (line_index + 1, offset - line_start + 1)
    }

    /// Get the byte offset of a line start (0-indexed line).
    #[inline]
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Get the number of lines.
    #[inline]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let index = LineIndex::from_source(b"");
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.line_column(0), (1, 1));
    }

    #[test]
    fn test_single_line() {
        let source = b"hello world";
        let index = LineIndex::from_source(source);
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.line_column(0), (1, 1));
        assert_eq!(index.line_column(5), (1, 6));
        assert_eq!(index.line_column(10), (1, 11));
    }

    #[test]
    fn test_multiple_lines() {
        let source = b"line1\nline2\nline3";
        let index = LineIndex::from_source(source);
        assert_eq!(index.line_count(), 3);

        // First line
        assert_eq!(index.line_column(0), (1, 1)); // 'l'
        assert_eq!(index.line_column(4), (1, 5)); // '1'

        // Second line
        assert_eq!(index.line_column(6), (2, 1)); // 'l' of line2
        assert_eq!(index.line_column(10), (2, 5)); // '2'

        // Third line
        assert_eq!(index.line_column(12), (3, 1)); // 'l' of line3
        assert_eq!(index.line_column(16), (3, 5)); // '3'
    }

    #[test]
    fn test_trailing_newline() {
        let source = b"line1\nline2\n";
        let index = LineIndex::from_source(source);
        assert_eq!(index.line_count(), 3);
        assert_eq!(index.line_column(12), (3, 1)); // Empty last line
    }

    #[test]
    fn test_at_newline() {
        let source = b"ab\ncd";
        let index = LineIndex::from_source(source);
        assert_eq!(index.line_column(2), (1, 3)); // '\n' character
        assert_eq!(index.line_column(3), (2, 1)); // 'c'
    }

    #[test]
    fn test_line_number_only() {
        let source = b"first\nsecond\nthird";
        let index = LineIndex::from_source(source);
        assert_eq!(index.line_number(0), 1);
        assert_eq!(index.line_number(5), 1); // '\n'
        assert_eq!(index.line_number(6), 2);
        assert_eq!(index.line_number(13), 3);
    }

    #[test]
    fn test_column_number_only() {
        let source = b"abc\ndefgh";
        let index = LineIndex::from_source(source);
        assert_eq!(index.column_number(0), 1);
        assert_eq!(index.column_number(2), 3);
        assert_eq!(index.column_number(4), 1); // 'd'
        assert_eq!(index.column_number(8), 5); // 'h'
    }
}
