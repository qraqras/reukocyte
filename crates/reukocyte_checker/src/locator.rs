/// LineIndex helps map byte offsets to line and column numbers.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
}
impl LineIndex {
    /// Build a LineIndex from source bytes.
    pub fn from_source(source: &[u8]) -> Self {
        let mut line_starts = Vec::with_capacity(source.len() / 80); // Rough estimate
        line_starts.push(0);
        for (i, &byte) in source.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }
    /// Get the line number (1-indexed) for a byte offset.
    pub fn line_number(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line + 1,
            Err(line) => line,
        }
    }
    /// Get the column number (1-indexed) for a byte offset.
    pub fn column_number(&self, offset: usize) -> usize {
        let line_index = self.line_number(offset) - 1;
        let line_start = self.line_starts[line_index];
        offset - line_start + 1
    }
    /// Get both line and column (1-indexed) for a byte offset.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = self.line_number(offset) - 1;
        let line_start = self.line_starts[line_index];
        (line_index + 1, offset - line_start + 1)
    }

    /// Batch resolve sorted offsets to (line, column) pairs.
    /// Optimized for sequential access - O(n) instead of O(n log n).
    #[inline]
    pub fn batch_line_column(&self, offsets: &[(usize, usize)]) -> Vec<(usize, usize, usize, usize)> {
        let mut results = Vec::with_capacity(offsets.len());
        let mut current_line_idx = 0;
        let line_count = self.line_starts.len();

        for &(start, end) in offsets {
            // Advance to the correct line for start offset
            while current_line_idx + 1 < line_count
                && self.line_starts[current_line_idx + 1] <= start
            {
                current_line_idx += 1;
            }

            let line_start_offset = self.line_starts[current_line_idx];
            let line_start = current_line_idx + 1;
            let column_start = start - line_start_offset + 1;

            // Find line for end offset (usually same line or close)
            let mut end_line_idx = current_line_idx;
            while end_line_idx + 1 < line_count
                && self.line_starts[end_line_idx + 1] <= end
            {
                end_line_idx += 1;
            }

            let end_line_start_offset = self.line_starts[end_line_idx];
            let line_end = end_line_idx + 1;
            let column_end = end - end_line_start_offset + 1;

            results.push((line_start, line_end, column_start, column_end));
        }

        results
    }

    /// Get the byte offset of a line start (0-indexed line).
    pub fn line_start(&self, line_index: usize) -> Option<usize> {
        self.line_starts.get(line_index).copied()
    }
    /// Get the number of lines.
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
