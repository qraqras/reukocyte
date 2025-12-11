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
use crate::diagnostic::{Edit, Fix};
use crate::rule::{Check, LayoutRule, Line, Rule, RuleId};
use reukocyte_macros::check;

/// Rule identifier for Layout/IndentationStyle.
pub struct IndentationStyle;

impl Rule for IndentationStyle {
    const ID: RuleId = RuleId::Layout(LayoutRule::IndentationStyle);
}

/// Check for tab indentation (default: spaces preferred).
#[check(Line)]
impl Check<Line<'_>> for IndentationStyle {
    fn check(line: &Line, checker: &mut Checker) {
        let config = &checker.config().layout.indentation_style;
        if !config.base.enabled {
            return;
        }
        if !checker.should_run_cop_cached("layout.indentation_style", &config.base) {
            return;
        }
        let severity = config.base.severity;

        if let Some((tab_start, tab_end)) = find_leading_tabs(line.text) {
            let abs_start = line.start + tab_start;
            let abs_end = line.start + tab_end;
            let tab_count = tab_end - tab_start;
            let replacement = " ".repeat(tab_count * 2);
            let fix = Fix::safe(vec![Edit::replacement(abs_start, abs_end, replacement)]);
            checker.report(Self::ID, "Tab detected in indentation.".to_string(), severity, abs_start, abs_end, Some(fix));
        }
    }
}

// NOTE: Now operating per-line using `Check<Line>`, so collect_edit_ranges is unused.

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
        assert!(diagnostics.is_empty(), "Expected no diagnostics for space indentation");
    }

    #[test]
    fn test_single_tab_indent() {
        let source = b"def foo\n\tbar\nend\n";
        let diagnostics = check(source);
        // IndentationStyle detects tab, IndentationWidth detects wrong indentation
        let style_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert_eq!(style_diagnostics.len(), 1);
        assert_eq!(style_diagnostics[0].message, "Tab detected in indentation.");
        // Tab is at position 8 (after "def foo\n")
        assert_eq!(style_diagnostics[0].start, 8);
        assert_eq!(style_diagnostics[0].end, 9);
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
        assert!(tab_violations.is_empty(), "Tab after code should not be flagged");
    }

    #[test]
    fn test_multiple_lines_with_tabs() {
        let source = b"def foo\n\tbar\n\tbaz\nend\n";
        let diagnostics = check(source);
        // Filter only IndentationStyle diagnostics
        let style_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.rule_id == RuleId::Layout(LayoutRule::IndentationStyle))
            .collect();
        assert_eq!(style_diagnostics.len(), 2);
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
