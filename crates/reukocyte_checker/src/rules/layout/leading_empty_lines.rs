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
use crate::rule::{Check, LayoutRule, Line, Rule, RuleId};
use reukocyte_macros::check;

/// LeadingEmptyLines rule as a struct so it can implement `Rule` + `Check<Line>`.
pub struct LeadingEmptyLines;

impl Rule for LeadingEmptyLines {
    const ID: RuleId = RuleId::Layout(LayoutRule::LeadingEmptyLines);
}

/// Check for leading empty lines in the source.
#[check(Line)]
impl Check<Line<'_>> for LeadingEmptyLines {
    fn check(line: &Line, checker: &mut Checker) {
        let config = &checker.config().layout.leading_empty_lines;
        if !config.base.enabled { return; }
        if !checker.should_run_cop(&config.base.include, &config.base.exclude) { return; }
        let severity = config.base.severity;

        // Trigger only when we encounter the first non-empty line.
        let is_non_empty = !line.text.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
        if !is_non_empty { return; }
        if line.index == 0 { return; }

        // Ensure all preceding lines are blank/whitespace-only.
        for j in 0..line.index {
            let prev = checker.line_index().line(j).unwrap_or(&[]);
            let prev_empty = prev.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
            if !prev_empty { return; }
        }

        // Count leading blank lines
        let leading_count = line.index;
        let end = line.start; // delete from 0 up to the start of this line

        let message = if leading_count == 1 {
            "Unnecessary blank line at the beginning of the source.".to_string()
        } else {
            format!("Unnecessary blank lines at the beginning of the source ({} lines).", leading_count)
        };

        let fix = Fix::safe(vec![Edit::deletion(0, end)]);
        checker.report(Self::ID, message, severity, 0, end, Some(fix));
    }
}

/// Analyze source for leading empty lines.
/// Returns (end_offset, message) if there's an issue.
// NOTE: `analyze` removed; logic is implemented per-line in Check<Line> above.

// Helper function `is_leading_whitespace` removed - not needed for per-line checks.

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
