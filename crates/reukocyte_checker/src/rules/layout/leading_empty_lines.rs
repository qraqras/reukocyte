//! Layout/LeadingEmptyLines
//!
//! Checks for unnecessary leading blank lines at the beginning of a file.
//!
//! # Examples
//!
//! ```ruby
//! # bad
//!
//! class Foo
//! end
//!
//! # good
//! class Foo
//! end
//! ```

use crate::Checker;
use crate::Edit;
use crate::Fix;
use crate::rule::{LayoutRule, RuleId};

/// Rule identifier for Layout/LeadingEmptyLines.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::LeadingEmptyLines);

/// Check for leading empty lines in the source.
pub fn check(checker: &mut Checker) {
    let config = &checker.config().layout.leading_empty_lines;
    if !config.enabled {
        return;
    }
    let severity = config.severity;

    if let Some((end, message)) = analyze(checker.source()) {
        let fix = Fix::safe(vec![Edit::deletion(0, end)]);
        checker.report(RULE_ID, message, severity, 0, end, Some(fix));
    }
}

/// Analyze source for leading empty lines.
/// Returns (end_offset, message) if there's an issue.
fn analyze(source: &[u8]) -> Option<(usize, String)> {
    if source.is_empty() {
        return None;
    }

    // Find the first non-whitespace character
    let first_content = source.iter().position(|&b| !is_leading_whitespace(b))?;

    // Count leading newlines
    let leading_newlines = source[..first_content].iter().filter(|&&b| b == b'\n').count();

    if leading_newlines > 0 {
        // Find the end of leading blank lines (position after the last leading newline)
        let end = source[..first_content].iter().rposition(|&b| b == b'\n').map(|pos| pos + 1).unwrap_or(0);

        let message = if leading_newlines == 1 {
            "Unnecessary blank line at the beginning of the source.".to_string()
        } else {
            format!("Unnecessary blank lines at the beginning of the source ({} lines).", leading_newlines)
        };

        Some((end, message))
    } else {
        None
    }
}

/// Check if a byte is leading whitespace (space, tab, newline, CR).
#[inline]
fn is_leading_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

#[cfg(test)]
mod tests {
    use crate::check;

    #[test]
    fn test_no_leading_empty_lines() {
        let source = b"class Foo\nend\n";
        let diagnostics = check(source);
        let leading = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").count();
        assert_eq!(leading, 0);
    }

    #[test]
    fn test_one_leading_empty_line() {
        let source = b"\nclass Foo\nend\n";
        let diagnostics = check(source);
        let leading: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").collect();
        assert_eq!(leading.len(), 1);
        assert!(leading[0].message.contains("Unnecessary blank line"));
    }

    #[test]
    fn test_multiple_leading_empty_lines() {
        let source = b"\n\n\nclass Foo\nend\n";
        let diagnostics = check(source);
        let leading: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").collect();
        assert_eq!(leading.len(), 1);
        assert!(leading[0].message.contains("3 lines"));
    }

    #[test]
    fn test_leading_spaces_only() {
        // Spaces at the beginning without newlines are not leading empty lines
        let source = b"  class Foo\nend\n";
        let diagnostics = check(source);
        let leading = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").count();
        assert_eq!(leading, 0);
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check(source);
        let leading = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").count();
        assert_eq!(leading, 0);
    }

    #[test]
    fn test_comment_at_start() {
        let source = b"# frozen_string_literal: true\nclass Foo\nend\n";
        let diagnostics = check(source);
        let leading = diagnostics.iter().filter(|d| d.rule() == "Layout/LeadingEmptyLines").count();
        assert_eq!(leading, 0);
    }
}
