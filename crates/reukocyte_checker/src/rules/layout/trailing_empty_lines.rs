//! Layout/TrailingEmptyLines
//!
//! Looks for trailing blank lines and a final newline in the source code.
//!
//! # Examples
//!
//! ```ruby
//! # bad - multiple trailing blank lines
//! class Foo
//! end
//!
//!
//! # good - single final newline
//! class Foo
//! end
//! ```
//!
//! # Configuration
//!
//! - `EnforcedStyle`: `final_newline` (default) or `final_blank_line`

use crate::Checker;
use crate::Edit;
use crate::Fix;
use crate::rule::{LayoutRule, RuleId};
use reukocyte_macros::check;

/// Rule identifier for Layout/TrailingEmptyLines.
pub const RULE_ID: RuleId = RuleId::Layout(LayoutRule::TrailingEmptyLines);

/// Enforced style for trailing empty lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnforcedStyle {
    /// Require exactly one final newline (no trailing blank lines).
    #[default]
    FinalNewline,
    /// Require one blank line followed by a final newline.
    FinalBlankLine,
}

/// Check for trailing empty lines in the source.
#[check(File)]
pub fn check(checker: &mut Checker) {
    let config = &checker.config().layout.trailing_empty_lines;
    if !config.base.enabled {
        return;
    }
    // Check cop-specific include/exclude
    if !checker.should_run_cop_cached("layout.trailing_empty_lines", &config.base) {
        return;
    }
    let severity = config.base.severity;

    // Convert config style to local enum
    let style = match config.enforced_style {
        crate::config::layout::trailing_empty_lines::EnforcedStyle::FinalNewline => EnforcedStyle::FinalNewline,
        crate::config::layout::trailing_empty_lines::EnforcedStyle::FinalBlankLine => EnforcedStyle::FinalBlankLine,
    };

    if let Some((start, end, replacement, message)) = analyze(checker.source(), style) {
        let fix = Fix::safe(vec![Edit::replacement(start, end, replacement)]);
        checker.report(RULE_ID, message, severity, start, end, Some(fix));
    }
}

/// Analyze source for trailing empty lines issues.
/// Returns (start, end, replacement, message) if there's an issue.
fn analyze(source: &[u8], style: EnforcedStyle) -> Option<(usize, usize, String, String)> {
    if source.is_empty() {
        return None;
    }

    // Count trailing newlines
    let trailing_newlines = source.iter().rev().take_while(|&&b| b == b'\n').count();

    // Find where trailing whitespace starts (including blank lines)
    let trailing_start = find_trailing_whitespace_start(source);

    let wanted_newlines = match style {
        EnforcedStyle::FinalNewline => 1,
        EnforcedStyle::FinalBlankLine => 2,
    };

    // Check for missing final newline
    if trailing_newlines == 0 {
        let message = "Final newline missing.".to_string();
        let replacement = "\n".repeat(wanted_newlines);
        return Some((source.len(), source.len(), replacement, message));
    }

    // Count blank lines (trailing_newlines - 1 because last \n is the final newline)
    let blank_lines = trailing_newlines.saturating_sub(1);
    let wanted_blank_lines = wanted_newlines.saturating_sub(1);

    if blank_lines != wanted_blank_lines {
        let message = if wanted_blank_lines == 0 {
            if blank_lines == 1 {
                "1 trailing blank line detected.".to_string()
            } else {
                format!("{} trailing blank lines detected.", blank_lines)
            }
        } else if blank_lines == 0 {
            "Trailing blank line missing.".to_string()
        } else {
            format!("{} trailing blank lines instead of {} detected.", blank_lines, wanted_blank_lines)
        };

        let replacement = "\n".repeat(wanted_newlines);
        return Some((trailing_start, source.len(), replacement, message));
    }

    None
}

/// Find the start position of trailing whitespace/blank lines.
fn find_trailing_whitespace_start(source: &[u8]) -> usize {
    let mut pos = source.len();

    for &byte in source.iter().rev() {
        if byte == b'\n' || byte == b' ' || byte == b'\t' || byte == b'\r' {
            pos -= 1;
        } else {
            break;
        }
    }

    // Include the final newline of the last content line
    if pos < source.len() && source.get(pos) == Some(&b'\n') {
        pos += 1;
    }

    pos
}

#[cfg(test)]
mod tests {
    use crate::check;

    #[test]
    fn test_final_newline_ok() {
        let source = b"class Foo\nend\n";
        let diagnostics = check(source);
        let trailing = diagnostics.iter().filter(|d| d.rule() == "Layout/TrailingEmptyLines").count();
        assert_eq!(trailing, 0);
    }

    #[test]
    fn test_missing_final_newline() {
        let source = b"class Foo\nend";
        let diagnostics = check(source);
        let trailing: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/TrailingEmptyLines").collect();
        assert_eq!(trailing.len(), 1);
        assert!(trailing[0].message.contains("Final newline missing"));
    }

    #[test]
    fn test_one_trailing_blank_line() {
        let source = b"class Foo\nend\n\n";
        let diagnostics = check(source);
        let trailing: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/TrailingEmptyLines").collect();
        assert_eq!(trailing.len(), 1);
        assert!(trailing[0].message.contains("1 trailing blank line"));
    }

    #[test]
    fn test_multiple_trailing_blank_lines() {
        let source = b"class Foo\nend\n\n\n";
        let diagnostics = check(source);
        let trailing: Vec<_> = diagnostics.iter().filter(|d| d.rule() == "Layout/TrailingEmptyLines").collect();
        assert_eq!(trailing.len(), 1);
        assert!(trailing[0].message.contains("2 trailing blank lines"));
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check(source);
        let trailing = diagnostics.iter().filter(|d| d.rule() == "Layout/TrailingEmptyLines").count();
        assert_eq!(trailing, 0);
    }
}
