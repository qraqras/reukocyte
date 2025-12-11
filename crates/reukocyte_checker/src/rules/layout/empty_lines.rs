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
use crate::rule::{Check, LayoutRule, Line, Rule, RuleId};
use reukocyte_macros::check;

/// Rule identifier for Layout/EmptyLines.
pub struct EmptyLines;

impl Rule for EmptyLines {
    const ID: RuleId = RuleId::Layout(LayoutRule::EmptyLines);
}

/// Check for consecutive empty lines in the source.
#[check(Line)]
impl Check<Line<'_>> for EmptyLines {
    fn check(line: &Line, checker: &mut Checker) {
        let config = &checker.config().layout.empty_lines;
        if !config.base.enabled {
            return;
        }
        if !checker.should_run_cop(&config.base.include, &config.base.exclude) {
            return;
        }
        let severity = config.base.severity;

        // Check if current line is empty (spaces, tabs, CR)
        let is_empty = line.text.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
        if !is_empty || line.index == 0 {
            return;
        }

        // Check previous line
        let prev_idx = line.index - 1;
        let prev_text = checker.line_index().line(prev_idx).unwrap_or(&[]);
        let prev_is_empty = prev_text.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
        if !prev_is_empty {
            return;
        }

        // Only report at the second consecutive empty line (avoid duplicates)
        if prev_idx > 0 {
            let prev_prev_text = checker.line_index().line(prev_idx - 1).unwrap_or(&[]);
            let prev_prev_empty = prev_prev_text.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
            if prev_prev_empty {
                // already reported at an earlier line
                return;
            }
        }

        // At this point, prev line is first empty, and current is second empty in a block
        // Find deletion range: from after first empty line's newline to the start of next non-empty line
        let mut first_empty_end = checker.line_index().line_start(prev_idx).unwrap_or(0) + prev_text.len();
        // Include newline if present
        if first_empty_end < checker.source().len() && checker.source()[first_empty_end] == b'\n' {
            first_empty_end += 1;
        }

        // Find end offset by scanning forward until next non-empty line
        let mut j = line.index + 1;
        let line_count = checker.line_index().line_count();
        while j < line_count {
            let nt = checker.line_index().line(j).unwrap_or(&[]);
            let nt_empty = nt.iter().all(|&b| b == b' ' || b == b'\t' || b == b'\r');
            if !nt_empty { break; }
            j += 1;
        }
        let end_offset = if j < line_count { checker.line_index().line_start(j).unwrap_or(checker.source().len()) } else { checker.source().len() };

        let extra_lines = j.saturating_sub(prev_idx + 1);
        let message = if extra_lines == 1 {
            "Extra blank line detected.".to_string()
        } else {
            format!("Extra blank line detected.")
        };
        let fix = Fix::safe(vec![Edit::deletion(first_empty_end, end_offset)]);
        checker.report(Self::ID, message, severity, first_empty_end, end_offset, Some(fix));
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

                let _extra_lines = consecutive_empty - 1;
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
