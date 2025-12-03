//! Layout/IndentationStyle
//!
//! Checks that indentation uses the configured style (spaces or tabs).
//! Default style is spaces.
//!
//! ## Examples
//!
//! ```ruby
//! # bad (with EnforcedStyle: spaces)
//! def foo
//! 	bar  # tab indentation
//! end
//!
//! # good
//! def foo
//!   bar  # space indentation
//! end
//! ```

use crate::checker::Checker;
use crate::diagnostic::{Edit, Fix, Severity};
use crate::rule::{LayoutRule, RuleId};

/// Rule identifier for Layout/IndentationStyle.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::IndentationStyle);

/// Check for tab indentation (default: spaces preferred).
pub fn check(checker: &mut Checker) {
    let edit_ranges = collect_edit_ranges(checker.source());
    for (start, end, replacement) in edit_ranges {
        let fix = Fix::safe(vec![Edit::replacement(start, end, replacement)]);
        checker.report(
            RULE_ID,
            "Tab detected in indentation.".to_string(),
            Severity::Convention,
            start,
            end,
            Some(fix),
        );
    }
}

/// Collect ranges of leading tabs and their replacements.
/// Returns (start, end, replacement) for each range of tabs found.
fn collect_edit_ranges(source: &[u8]) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();

    // Empty source, nothing to check
    if source.is_empty() {
        return ranges;
    }

    // Process each line
    let mut pos = 0;
    for line in source.split(|&b| b == b'\n') {
        // Find leading tabs in this line
        if let Some((tab_start, tab_end)) = find_leading_tabs(line) {
            let abs_start = pos + tab_start;
            let abs_end = pos + tab_end;

            // Calculate replacement spaces (2 spaces per tab, configurable later)
            let tab_count = tab_end - tab_start;
            let replacement = " ".repeat(tab_count * 2);

            ranges.push((abs_start, abs_end, replacement));
        }

        // Move to next line (+1 for newline character)
        pos += line.len() + 1;
    }

    ranges
}

/// Find the range of leading tabs in a line.
/// Returns (start, end) offsets relative to line start, or None if no leading tabs.
fn find_leading_tabs(line: &[u8]) -> Option<(usize, usize)> {
    let mut start = None;
    let mut end = 0;

    for (i, &b) in line.iter().enumerate() {
        match b {
            b'\t' => {
                if start.is_none() {
                    start = Some(i);
                }
                end = i + 1;
            }
            b' ' => {
                // Allow mixed leading whitespace, continue
                continue;
            }
            _ => {
                // Non-whitespace, stop
                break;
            }
        }
    }

    start.map(|s| (s, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check;
    use crate::diagnostic::Applicability;

    #[test]
    fn test_no_tabs() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(
            diagnostics.is_empty(),
            "Expected no diagnostics for space indentation"
        );
    }

    #[test]
    fn test_single_tab_indent() {
        let source = b"def foo\n\tbar\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].rule_id,
            RuleId::Layout(LayoutRule::IndentationStyle)
        );
        assert_eq!(diagnostics[0].message, "Tab detected in indentation.");
        // Tab is at position 8 (after "def foo\n")
        assert_eq!(diagnostics[0].start, 8);
        assert_eq!(diagnostics[0].end, 9);
    }

    #[test]
    fn test_multiple_tabs() {
        let source = b"def foo\n\t\tbar\nend\n";
        let diagnostics = check(source);
        let style_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert_eq!(style_diags.len(), 1);
        // Two tabs
        assert_eq!(style_diags[0].start, 8);
        assert_eq!(style_diags[0].end, 10);
    }

    #[test]
    fn test_tab_with_fix() {
        let source = b"def foo\n\tbar\nend\n";
        let diagnostics = check(source);
        let style_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert_eq!(style_diags.len(), 1);

        let fix = style_diags[0].fix.as_ref().unwrap();
        assert_eq!(fix.applicability, Applicability::Safe);
        assert_eq!(fix.edits.len(), 1);
        // Single tab -> 2 spaces
        assert_eq!(fix.edits[0].content, "  ");
    }

    #[test]
    fn test_two_tabs_with_fix() {
        let source = b"def foo\n\t\tbar\nend\n";
        let diagnostics = check(source);
        let style_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();

        let fix = style_diags[0].fix.as_ref().unwrap();
        // Two tabs -> 4 spaces
        assert_eq!(fix.edits[0].content, "    ");
    }

    #[test]
    fn test_mixed_space_then_tab() {
        let source = b"def foo\n  \tbar\nend\n";
        let diagnostics = check(source);
        let style_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert_eq!(style_diags.len(), 1);
        // Tab at position 10 (after "def foo\n  ")
        assert_eq!(style_diags[0].start, 10);
        assert_eq!(style_diags[0].end, 11);
    }

    #[test]
    fn test_tab_not_at_start_of_line() {
        // Tab after code should not be flagged (it's alignment, not indentation)
        let source = b"def foo\n  bar\tbaz\nend\n";
        let diagnostics = check(source);
        let tab_violations: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert!(
            tab_violations.is_empty(),
            "Tab after code should not be flagged"
        );
    }

    #[test]
    fn test_multiple_lines_with_tabs() {
        let source = b"def foo\n\tbar\n\tbaz\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check(source);
        let tab_violations: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert!(tab_violations.is_empty());
    }
}
