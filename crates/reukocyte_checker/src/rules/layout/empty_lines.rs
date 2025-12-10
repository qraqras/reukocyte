//! Layout/EmptyLines
//!
//! Checks for two or more consecutive blank lines.
//!
//! # Examples
//!
//! ```ruby
//! # bad - two empty lines
//! def foo
//! end
//!
//!
//! def bar
//! end
//!
//! # good
//! def foo
//! end
//!
//! def bar
//! end
//! ```

use crate::Checker;
use crate::Edit;
use crate::Fix;
use crate::rule::{LayoutRule, RuleId};

/// Rule identifier for Layout/EmptyLines.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::EmptyLines);

/// Check for consecutive empty lines in the source.
pub fn check(checker: &mut Checker) {
    let config = &checker.config().layout.empty_lines;
    if !config.base.enabled {
        return;
    }
    // Check cop-specific include/exclude
    if !checker.should_run_cop(&config.base.include, &config.base.exclude) {
        return;
    }
    let severity = config.base.severity;

    let edit_ranges = collect_edit_ranges(checker.source());
    for (start, end, message) in edit_ranges {
        // Fix: remove extra blank lines, keeping just one
        let fix = Fix::safe(vec![Edit::deletion(start, end)]);
        checker.report(RULE_ID, message, severity, start, end, Some(fix));
    }
}

/// Collect ranges of extra blank lines.
/// Returns (start, end, message) for each occurrence.
fn collect_edit_ranges(source: &[u8]) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    let mut offset = 0;
    let mut consecutive_empty = 0;
    let mut empty_start = 0;

    for line in source.split(|&b| b == b'\n') {
        let is_empty = line.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');

        if is_empty {
            if consecutive_empty == 0 {
                // First empty line - remember where it started
                empty_start = offset;
            }
            consecutive_empty += 1;
        } else {
            if consecutive_empty >= 2 {
                // We had 2+ consecutive empty lines
                // Keep one empty line, delete the rest
                // The range to delete starts after the first empty line
                let first_empty_end = find_first_newline_after(source, empty_start).map(|pos| pos + 1).unwrap_or(empty_start);

                let extra_lines = consecutive_empty - 1;
                let message = format!("Extra blank line detected.");

                // Delete from after first empty line to start of current line
                if first_empty_end < offset {
                    ranges.push((first_empty_end, offset, message));
                }
            }
            consecutive_empty = 0;
        }

        offset += line.len() + 1; // +1 for '\n'
    }

    // Handle trailing empty lines (but don't duplicate TrailingEmptyLines)
    // We only report if there are 2+ consecutive empty lines in the middle

    ranges
}

/// Find the position of the first newline at or after the given position.
fn find_first_newline_after(source: &[u8], start: usize) -> Option<usize> {
    source[start..].iter().position(|&b| b == b'\n').map(|pos| start + pos)
}

#[cfg(test)]
mod tests {
    use crate::check;

    #[test]
    fn test_no_consecutive_empty_lines() {
        let source = b"def foo\nend\n\ndef bar\nend\n";
        let diagnostics = check(source);
        let empty = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").count();
        assert_eq!(empty, 0);
    }

    #[test]
    fn test_two_consecutive_empty_lines() {
        let source = b"def foo\nend\n\n\ndef bar\nend\n";
        let diagnostics = check(source);
        let empty: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").collect();
        assert_eq!(empty.len(), 1);
        assert!(empty[0].message.contains("Extra blank line"));
    }

    #[test]
    fn test_three_consecutive_empty_lines() {
        let source = b"def foo\nend\n\n\n\ndef bar\nend\n";
        let diagnostics = check(source);
        let empty: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").collect();
        assert_eq!(empty.len(), 1);
    }

    #[test]
    fn test_multiple_occurrences() {
        let source = b"def foo\nend\n\n\ndef bar\nend\n\n\ndef baz\nend\n";
        let diagnostics = check(source);
        let empty: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").collect();
        assert_eq!(empty.len(), 2);
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check(source);
        let empty = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").count();
        assert_eq!(empty, 0);
    }

    #[test]
    fn test_single_empty_line_ok() {
        let source = b"class Foo\n\n  def bar\n  end\nend\n";
        let diagnostics = check(source);
        let empty = diagnostics.iter().filter(|d| d.rule() == "Layout/EmptyLines").count();
        assert_eq!(empty, 0);
    }
}
