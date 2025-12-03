use crate::Checker;
use crate::Edit;
use crate::Fix;
use crate::Severity;
use crate::rule::{LayoutRule, RuleId};
use crate::utils::is_blank;

/// Rule identifier for Layout/TrailingWhitespace.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::TrailingWhitespace);

/// Check for trailing whitespace in the source.
///
/// This rule doesn't need AST information - it operates on raw source bytes.
/// Directly pushes diagnostics to the Checker (Ruff-style).
pub fn check(checker: &mut Checker) {
    // Collect edit ranges first, then report them
    let edit_ranges = collect_edit_ranges(checker.source());
    for (start, end) in edit_ranges {
        let fix = Fix::safe(vec![Edit::deletion(start, end)]);
        checker.report(
            RULE_ID,
            "Trailing whitespace detected.".to_string(),
            Severity::Convention,
            start,
            end,
            Some(fix),
        );
    }
}

/// Collect (start, end) byte offsets of trailing whitespace.
fn collect_edit_ranges(source: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut offset = 0;
    for line in source.split(|&b| b == b'\n') {
        if let Some(trailing_start) = find_trailing_whitespace(line) {
            let start = offset + trailing_start;
            let end = offset + line.len();
            ranges.push((start, end));
        }
        offset += line.len() + 1;
    }
    ranges
}

/// Find the start position of trailing whitespace in a line.
/// Returns `None` if there is no trailing whitespace or invalid UTF-8.
fn find_trailing_whitespace(line: &[u8]) -> Option<usize> {
    let line_str = std::str::from_utf8(line).ok()?;

    let trimmed_len = line_str
        .char_indices()
        .rev()
        .find(|(_, c)| !is_blank(*c))
        .map(|(pos, c)| pos + c.len_utf8())
        .unwrap_or(0);

    if trimmed_len < line.len() && !line.is_empty() {
        Some(trimmed_len)
    } else {
        None
    }
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
