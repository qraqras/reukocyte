/// LineIndex helps map byte offsets to line and column numbers.
#[derive(Debug, Clone)]
pub struct LineIndex<'rk> {
    line_starts: Vec<usize>,
    lines: Vec<&'rk [u8]>,
}
impl<'rk> LineIndex<'rk> {
    /// Build a LineIndex from source bytes.
    pub fn from_source(source: &'rk [u8]) -> Self {
        let mut line_starts = Vec::with_capacity(source.len() / 80); // Rough estimate
        line_starts.push(0);
        for (i, &byte) in source.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(i + 1);
            }
        }

        // Build lines cache
        let mut lines = Vec::with_capacity(line_starts.len());
        for i in 0..line_starts.len() {
            let start = line_starts[i];
            let end = if i + 1 < line_starts.len() {
                line_starts[i + 1].saturating_sub(1)
            } else {
                source.len()
            };
            lines.push(&source[start..end]);
        }

        Self { line_starts, lines }
    }
    /// Get the line index (0-indexed) for a byte offset.
    pub fn line_index(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        }
    }
    /// Get the line number (1-indexed) for a byte offset.
    pub fn line_number(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line.saturating_add(1),
            Err(line) => line,
        }
    }
    /// Get the column number (1-indexed) for a byte offset.
    pub fn column_number(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        let line_start = self.line_starts[line_index];
        offset - line_start + 1
    }
    /// Get both line and column (1-indexed) for a byte offset.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = self.line_index(offset);
        let line_start = self.line_starts[line_index];
        (line_index + 1, offset - line_start + 1)
    }

    /// Get the byte range for the line containing the given offset.
    /// Returns (line_start, next_line_start) where next_line_start is None for the last line.
    pub fn line_range(&self, offset: usize) -> (usize, Option<usize>) {
        let line_index = self.line_index(offset);
        let line_start = self.line_starts[line_index];
        let next_line_start = self.line_starts.get(line_index + 1).copied();
        (line_start, next_line_start)
    }

    /// Check if two byte offsets are on the same line.
    /// Optimized to use single binary search.
    pub fn are_on_same_line(&self, pos1: usize, pos2: usize) -> bool {
        let (curr_line_start, next_line_start) = self.line_range(pos1);
        curr_line_start <= pos2 && next_line_start.map_or(true, |next| pos2 < next)
    }

    /// Get the line content for a given line index (0-indexed).
    pub fn line(&self, line_index: usize) -> Option<&'rk [u8]> {
        self.lines.get(line_index).copied()
    }

    /// Get the line content for a given byte offset.
    pub fn line_at(&self, offset: usize) -> &'rk [u8] {
        let line_index = self.line_index(offset);
        self.lines[line_index]
    }

    /// Check if the offset is at the beginning of its line (ignoring leading whitespace).
    pub fn is_first_on_line(&self, offset: usize) -> bool {
        let line_index = self.line_index(offset);
        let line_start = self.line_starts[line_index];
        let prefix = &self.lines[line_index][..offset - line_start];
        prefix.iter().all(|&b| b == b' ' || b == b'\t')
    }

    /// Calculate the column offset from pos1 to pos2 (can be negative if pos2 is to the left of pos1).
    pub fn column_offset_between(&self, pos1: usize, pos2: usize) -> i32 {
        let col1 = self.column_number(pos1) as i32;
        let col2 = self.column_number(pos2) as i32;
        col2 - col1
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
            while current_line_idx + 1 < line_count && self.line_starts[current_line_idx + 1] <= start {
                current_line_idx += 1;
            }

            let line_start_offset = self.line_starts[current_line_idx];
            let line_start = current_line_idx + 1;
            let column_start = start - line_start_offset + 1;

            // Find line for end offset (usually same line or close)
            let mut end_line_idx = current_line_idx;
            while end_line_idx + 1 < line_count && self.line_starts[end_line_idx + 1] <= end {
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

    /// Get the byte offset where the line starts for a given byte offset.
    pub fn line_start_offset(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        self.line_starts[line_index]
    }

    /// Get the byte offset where the line ends (before newline) for a given byte offset.
    pub fn line_end_offset(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        if line_index + 1 < self.line_starts.len() {
            self.line_starts[line_index + 1].saturating_sub(1)
        } else {
            // Last line - return length of the line content
            self.line_starts[line_index] + self.lines[line_index].len()
        }
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    // ========================================================================
    // Indentation methods (counting leading whitespace)
    // ========================================================================

    /// Get the indentation level (0-indexed) at the start of the line containing the offset.
    ///
    /// This counts only leading whitespace characters (spaces and tabs).
    /// Each space or tab counts as 1 (tab width is not expanded).
    ///
    /// # Example
    /// ```ignore
    /// // "  def" -> indentation is 2
    /// // "\tdef" -> indentation is 1 (tab counts as 1)
    /// // "  \tdef" -> indentation is 3 (2 spaces + 1 tab)
    /// ```
    pub fn indentation(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        let line = self.lines[line_index];

        line.iter().take_while(|&&b| b == b' ' || b == b'\t').count()
    }

    /// Get the column position (0-indexed) within the line, counting each byte as 1.
    ///
    /// Unlike `column_number` which is 1-indexed, this returns 0-indexed position.
    /// Tabs are counted as 1 character (not expanded).
    pub fn column(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        let line_start = self.line_starts[line_index];
        offset - line_start
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

    // ========================================================================
    // Indentation tests
    // ========================================================================

    #[test]
    fn test_indentation_spaces() {
        let source = b"    def foo";
        let index = LineIndex::from_source(source);
        assert_eq!(index.indentation(0), 4);
        assert_eq!(index.indentation(4), 4); // offset within same line
    }

    #[test]
    fn test_indentation_tab() {
        let source = b"\tdef foo";
        let index = LineIndex::from_source(source);
        assert_eq!(index.indentation(0), 1); // tab counts as 1
    }

    #[test]
    fn test_indentation_mixed() {
        // "  \t  def" -> 2 spaces + 1 tab + 2 spaces = 5
        let source = b"  \t  def";
        let index = LineIndex::from_source(source);
        assert_eq!(index.indentation(0), 5);
    }

    #[test]
    fn test_indentation_no_indent() {
        let source = b"def foo";
        let index = LineIndex::from_source(source);
        assert_eq!(index.indentation(0), 0);
    }

    #[test]
    fn test_indentation_multiline() {
        let source = b"class Foo\n  def bar\n    x\n  end\nend";
        let index = LineIndex::from_source(source);
        assert_eq!(index.indentation(0), 0); // "class Foo"
        assert_eq!(index.indentation(10), 2); // "  def bar"
        assert_eq!(index.indentation(20), 4); // "    x"
        assert_eq!(index.indentation(26), 2); // "  end"
        assert_eq!(index.indentation(32), 0); // "end"
    }

    #[test]
    fn test_column_0indexed() {
        let source = b"  def foo";
        let index = LineIndex::from_source(source);
        assert_eq!(index.column(0), 0); // first space
        assert_eq!(index.column(2), 2); // 'd'
        assert_eq!(index.column(5), 5); // 'f'
    }

    #[test]
    fn test_column_multiline() {
        let source = b"abc\n  def";
        let index = LineIndex::from_source(source);
        assert_eq!(index.column(0), 0); // 'a'
        assert_eq!(index.column(2), 2); // 'c'
        assert_eq!(index.column(4), 0); // first space of line 2
        assert_eq!(index.column(6), 2); // 'd'
    }
}
