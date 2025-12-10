use crate::Checker;
use crate::Edit;
use crate::Fix;
use crate::rule::{LayoutRule, RuleId};

/// Rule identifier for Layout/TrailingWhitespace.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::TrailingWhitespace);

/// Check for trailing whitespace in the source.
///
/// This rule doesn't need AST information - it operates on raw source bytes.
/// Directly pushes diagnostics to the Checker (Ruff-style).
pub fn check(checker: &mut Checker) {
    let config = &checker.config().layout.trailing_whitespace;
    if !config.enabled {
        return;
    }
    let severity = config.severity;

    // Collect edit ranges first, then report them
    let edit_ranges = collect_edit_ranges(checker.source());
    for (start, end) in edit_ranges {
        let fix = Fix::safe(vec![Edit::deletion(start, end)]);
        checker.report(RULE_ID, "Trailing whitespace detected.".to_string(), severity, start, end, Some(fix));
    }
}

/// Collect (start, end) byte offsets of trailing whitespace.
fn collect_edit_ranges(source: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut offset = 0;
    for line in source.split(|&b| b == b'\n') {
        if let Some(trailing_start) = find_trailing_whitespace_fast(line) {
            let start = offset + trailing_start;
            let end = offset + line.len();
            ranges.push((start, end));
        }
        offset += line.len() + 1;
    }
    ranges
}

/// Fast byte-level detection of trailing whitespace.
/// RuboCop's [[:blank:]] = space (0x20), tab (0x09), fullwidth space (U+3000 = 0xE3 0x80 0x80)
///
/// Optimized: scan backwards byte-by-byte, only decode UTF-8 when needed.
#[inline]
fn find_trailing_whitespace_fast(line: &[u8]) -> Option<usize> {
    if line.is_empty() {
        return None;
    }

    let len = line.len();
    let mut pos = len;

    // Scan backwards
    while pos > 0 {
        let b = line[pos - 1];

        match b {
            // ASCII space or tab - continue scanning
            b' ' | b'\t' => {
                pos -= 1;
            }
            // Potential fullwidth space end byte (0x80)
            // Fullwidth space is U+3000 = 0xE3 0x80 0x80 in UTF-8
            0x80 if pos >= 3 && line[pos - 3] == 0xE3 && line[pos - 2] == 0x80 => {
                pos -= 3;
            }
            // Non-blank character found
            _ => break,
        }
    }

    if pos < len { Some(pos) } else { None }
}

#[cfg(test)]
mod tests {
    use crate::check;

    #[test]
    fn test_no_trailing_whitespace() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_trailing_spaces() {
        let source = b"def foo  \n  bar\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line_start, 1);
        assert_eq!(diagnostics[0].column_start, 8); // After "def foo"
    }

    #[test]
    fn test_trailing_tab() {
        let source = b"def foo\t\n  bar\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line_start, 1);
    }

    #[test]
    fn test_multiple_lines_with_trailing() {
        let source = b"def foo  \n  bar  \nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].line_start, 1);
        assert_eq!(diagnostics[1].line_start, 2);
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_whitespace_only_line() {
        let source = b"def foo\n   \nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line_start, 2);
    }

    #[test]
    fn test_fullwidth_space() {
        // Fullwidth space (U+3000) at end of line
        let source = "x = 0\u{3000}\n".as_bytes();
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line_start, 1);
        assert_eq!(diagnostics[0].column_start, 6); // After "x = 0"
    }

    #[test]
    fn test_cr_not_trailing_whitespace() {
        // CR should NOT be detected as trailing whitespace (RuboCop behavior)
        let source = b"def foo\r\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }
}
