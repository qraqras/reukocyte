/// A representation of a single source line for line-based checks.
///
/// `Line` contains the line index (0-based), the byte range of the line
/// relative to the source, and the raw line bytes.
#[derive(Debug, Clone)]
pub struct Line<'rk> {
    pub index: usize,
    pub start: usize,
    pub end: usize,
    pub indent: usize,
    pub text: &'rk [u8],
}

#[derive(Debug, Clone)]
pub struct LineIndex<'rk> {
    lines: Vec<Line<'rk>>,
    stride: usize,
    stride_starts: Vec<usize>,
}
impl<'rk> LineIndex<'rk> {
    /// Build a LineIndex from source bytes.
    pub fn from_source(source: &'rk [u8]) -> Self {
        // Collect line start and end offsets
        let mut line_starts = Vec::with_capacity(source.len() / 80);
        let mut line_ends = Vec::with_capacity(source.len() / 80);
        // First line starts at 0
        line_starts.push(0);
        for (pos, &b) in source.iter().enumerate() {
            if b == b'\n' {
                line_ends.push(pos);
                line_starts.push(pos + 1);
            }
        }
        // Last line ends at source.len()
        line_ends.push(source.len());

        // Build Line structures
        let mut lines = Vec::with_capacity(line_starts.len());
        for i in 0..line_starts.len() {
            let start = line_starts[i];
            let end = line_ends[i];
            let text = &source[start..end];
            let indent = text.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(text.len());
            lines.push(Line {
                index: i,
                start: start,
                end: end,
                indent: indent,
                text: text,
            });
        }

        // Build stride index for faster line lookup
        const STRIDE_SIZE: usize = 64;
        let mut stride_starts = Vec::new();
        for i in (0..lines.len()).step_by(STRIDE_SIZE) {
            stride_starts.push(lines[i].start);
        }

        Self {
            lines,
            stride: STRIDE_SIZE,
            stride_starts,
        }
    }
    /// Get all lines.
    pub fn lines(&self) -> &[Line<'rk>] {
        &self.lines
    }
    /// Get the line index (0-indexed) for a byte offset.
    pub fn line_index(&self, offset: usize) -> usize {
        // Use stride index for initial guess
        let stride_idx = match self.stride_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let mut guess = stride_idx * self.stride;
        if guess >= self.lines.len() {
            guess = self.lines.len().saturating_sub(1);
        }
        // Linear search within the stride group
        while guess + 1 < self.lines.len() && self.lines[guess + 1].start <= offset {
            guess += 1;
        }
        guess
    }
    /// Get the line number (1-indexed) for a byte offset.
    pub fn line_number(&self, offset: usize) -> usize {
        match self.lines.binary_search_by_key(&offset, |l| l.start) {
            Ok(line) => line.saturating_add(1),
            Err(line) => line,
        }
    }
    /// Get the column number (1-indexed) for a byte offset.
    pub fn column_number(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        let line_start = self.lines[line_index].start;
        offset - line_start + 1
    }
    /// Get both line and column (1-indexed) for a byte offset.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = self.line_index(offset);
        let line_start = self.lines[line_index].start;
        (line_index + 1, offset - line_start + 1)
    }

    /// Get the byte range for the line containing the given offset.
    /// Returns (line_start, next_line_start) where next_line_start is None for the last line.
    pub fn line_range(&self, offset: usize) -> (usize, Option<usize>) {
        let line_index = self.line_index(offset);
        let line_start = self.lines[line_index].start;
        let next_line_start = self.lines.get(line_index + 1).map(|l| l.start);
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
        self.lines.get(line_index).map(|l| l.text)
    }

    /// Get the line content for a given byte offset.
    pub fn line_at(&self, offset: usize) -> &'rk [u8] {
        let line_index = self.line_index(offset);
        self.lines[line_index].text
    }

    /// Check if the offset is at the beginning of its line (ignoring leading whitespace).
    pub fn is_first_on_line(&self, offset: usize) -> bool {
        let line_index = self.line_index(offset);
        let line_start = self.lines[line_index].start;
        let prefix = &self.lines[line_index].text[..offset - line_start];
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
        let mut current_line_idx = if !offsets.is_empty() { self.line_index(offsets[0].0) } else { 0 };
        let line_count = self.lines.len();
        let lines = &self.lines;

        for &(start, end) in offsets {
            // Advance to the correct line for start offset
            while current_line_idx + 1 < line_count && lines[current_line_idx + 1].start <= start {
                current_line_idx += 1;
            }
            let line_start_offset = lines[current_line_idx].start;
            let line_start = current_line_idx + 1;
            let column_start = start - line_start_offset + 1;

            // Find line for end offset (usually same line or close)
            let mut end_line_idx = current_line_idx;
            // Fast path: if end is before the next line start, remain on same line
            if end >= lines[end_line_idx].start {
                if end_line_idx + 1 < line_count {
                    let next_start = lines[end_line_idx + 1].start;
                    if end >= next_start {
                        while end_line_idx + 1 < line_count && lines[end_line_idx + 1].start <= end {
                            end_line_idx += 1;
                        }
                    }
                }
            }
            let end_line_start_offset = lines[end_line_idx].start;
            let line_end = end_line_idx + 1;
            let column_end = end - end_line_start_offset + 1;

            results.push((line_start, line_end, column_start, column_end));
        }

        results
    }

    /// Batch resolve sorted offsets to (line, column, indentation) tuples.
    /// Returns (line_start, line_end, column_start, column_end, indentation)
    /// where indentation is the number of leading whitespace chars on the start line.
    #[inline]
    pub fn batch_line_info(&self, offsets: &[(usize, usize)]) -> Vec<(usize, usize, usize, usize, usize)> {
        let mut results = Vec::with_capacity(offsets.len());
        let line_count = self.lines.len();
        let lines = &self.lines;
        // Start from the line index for the first offset to avoid scanning from 0
        let mut current_line_idx = if !offsets.is_empty() { self.line_index(offsets[0].0) } else { 0usize };
        // cache the indentation for the current_line_idx
        let mut current_indent = self.lines[current_line_idx].indent;

        for &(start, end) in offsets {
            while current_line_idx + 1 < line_count && self.lines[current_line_idx + 1].start <= start {
                current_line_idx += 1;
                current_indent = self.lines[current_line_idx].indent;
            }

            let line_start_offset = self.lines[current_line_idx].start;
            let line_start = current_line_idx + 1;
            let column_start = start - line_start_offset + 1;

            let mut end_line_idx = current_line_idx;
            // Fast path: if end is before the next line start, remain on same line
            if end >= lines[end_line_idx].start {
                if end_line_idx + 1 < line_count {
                    let next_start = lines[end_line_idx + 1].start;
                    if end >= next_start {
                        while end_line_idx + 1 < line_count && lines[end_line_idx + 1].start <= end {
                            end_line_idx += 1;
                        }
                    }
                }
            }
            let end_line_start_offset = self.lines[end_line_idx].start;
            let line_end = end_line_idx + 1;
            let column_end = end - end_line_start_offset + 1;

            results.push((line_start, line_end, column_start, column_end, current_indent));
        }

        results
    }

    /// Batch resolve sorted offsets into the provided output vector.
    /// This is a non-allocating version of `batch_line_info` which clears and
    /// fills `out` with `(line_start, line_end, column_start, column_end, indent)`.
    #[inline]
    pub fn batch_line_info_into(&self, offsets: &[(usize, usize)], out: &mut Vec<(usize, usize, usize, usize, usize)>) {
        out.clear();
        out.reserve(offsets.len());
        let lines = &self.lines;
        let line_count = lines.len();
        // Start from the line index for the first offset to avoid scanning from 0
        let mut current_line_idx = if !offsets.is_empty() { self.line_index(offsets[0].0) } else { 0usize };
        // cache the indentation for the current_line_idx
        let mut current_indent = lines[current_line_idx].indent;

        for &(start, end) in offsets {
            while current_line_idx + 1 < line_count && lines[current_line_idx + 1].start <= start {
                current_line_idx += 1;
                current_indent = lines[current_line_idx].indent;
            }

            let line_start_offset = lines[current_line_idx].start;
            let line_start = current_line_idx + 1;
            let column_start = start - line_start_offset + 1;

            let mut end_line_idx = current_line_idx;
            while end_line_idx + 1 < line_count && lines[end_line_idx + 1].start <= end {
                end_line_idx += 1;
            }
            let end_line_start_offset = lines[end_line_idx].start;
            let line_end = end_line_idx + 1;
            let column_end = end - end_line_start_offset + 1;

            out.push((line_start, line_end, column_start, column_end, current_indent));
        }
    }

    /// Get the byte offset of a line start (0-indexed line).
    pub fn line_start(&self, line_index: usize) -> Option<usize> {
        self.lines.get(line_index).map(|l| l.start)
    }

    /// Get the byte offset where the line starts for a given byte offset.
    pub fn line_start_offset(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        self.lines[line_index].start
    }

    /// Get the byte offset where the line ends (before newline) for a given byte offset.
    pub fn line_end_offset(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        if line_index + 1 < self.lines.len() {
            self.lines[line_index + 1].start.saturating_sub(1)
        } else {
            self.lines[line_index].end
        }
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
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
        self.lines[line_index].indent
    }

    /// Get the column position (0-indexed) within the line, counting each byte as 1.
    ///
    /// Unlike `column_number` which is 1-indexed, this returns 0-indexed position.
    /// Tabs are counted as 1 character (not expanded).
    pub fn column(&self, offset: usize) -> usize {
        let line_index = self.line_index(offset);
        let line_start = self.lines[line_index].start;
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

    #[test]
    fn test_batch_line_info_matches_individual() {
        // Build a source with many lines and varying indentation
        let mut src = Vec::new();
        for i in 0..1000 {
            if i % 3 == 0 {
                src.extend_from_slice(b"  line\n");
            } else if i % 3 == 1 {
                src.extend_from_slice(b"\tline\n");
            } else {
                src.extend_from_slice(b"line\n");
            }
        }
        let index = LineIndex::from_source(&src);
        // Prepare offsets (start,end) for some positions across the file
        let mut offsets = Vec::new();
        for line in (0..1000).step_by(7) {
            let start = index.line_start(line).unwrap();
            let end = start + 2; // span within the line
            offsets.push((start, end));
        }
        let batch = index.batch_line_info(&offsets);
        for (i, &(start, end)) in offsets.iter().enumerate() {
            let (ls, cs) = index.line_column(start);
            let (le, ce) = index.line_column(end);
            let indent = index.indentation(start);
            let (bls, ble, bcs, bce, bind) = batch[i];
            assert_eq!(ls, bls);
            assert_eq!(le, ble);
            assert_eq!(cs, bcs);
            assert_eq!(ce, bce);
            assert_eq!(indent, bind);
        }
    }
}
