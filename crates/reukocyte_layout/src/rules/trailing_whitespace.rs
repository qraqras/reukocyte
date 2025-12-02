//! Layout/TrailingWhitespace
//!
//! Detects trailing whitespace at the end of lines.
//!
//! # Examples
//!
//! ```ruby
//! # bad
//! def foo··
//!   bar
//! end
//!
//! # good
//! def foo
//!   bar
//! end
//! ```

use ruby_prism::ParseResult;

use crate::Diagnostic;

const RULE_NAME: &str = "Layout/TrailingWhitespace";

/// Check for trailing whitespace in the source.
///
/// This rule doesn't need AST information - it operates on raw source bytes.
pub fn check(source: &[u8], _parse_result: &ParseResult<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (line_index, line) in source.split(|&b| b == b'\n').enumerate() {
        if let Some(trailing_start) = find_trailing_whitespace(line) {
            let line_number = line_index + 1;
            let column = trailing_start + 1;

            // Calculate byte offset
            let line_start: usize = source
                .split(|&b| b == b'\n')
                .take(line_index)
                .map(|l| l.len() + 1)
                .sum();

            diagnostics.push(Diagnostic {
                rule: RULE_NAME,
                message: "Trailing whitespace detected.".to_string(),
                start: line_start + trailing_start,
                end: line_start + line.len(),
                line: line_number,
                column,
            });
        }
    }

    diagnostics
}

/// Find the start position of trailing whitespace in a line.
/// Returns `None` if there is no trailing whitespace.
fn find_trailing_whitespace(line: &[u8]) -> Option<usize> {
    let trimmed_len = line
        .iter()
        .rposition(|&b| b != b' ' && b != b'\t' && b != b'\r')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    if trimmed_len < line.len() && !line.is_empty() {
        // Check if the entire line is not just whitespace
        // (we don't flag blank lines with only whitespace in some configs,
        // but for now we flag all trailing whitespace)
        Some(trimmed_len)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(source: &[u8]) -> Vec<Diagnostic> {
        let parse_result = ruby_prism::parse(source);
        check(source, &parse_result)
    }

    #[test]
    fn test_no_trailing_whitespace() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_trailing_spaces() {
        let source = b"def foo  \n  bar\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 1);
        assert_eq!(diagnostics[0].column, 8); // After "def foo"
    }

    #[test]
    fn test_trailing_tab() {
        let source = b"def foo\t\n  bar\nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 1);
    }

    #[test]
    fn test_multiple_lines_with_trailing() {
        let source = b"def foo  \n  bar  \nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].line, 1);
        assert_eq!(diagnostics[1].line, 2);
    }

    #[test]
    fn test_empty_file() {
        let source = b"";
        let diagnostics = check_source(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_whitespace_only_line() {
        let source = b"def foo\n   \nend\n";
        let diagnostics = check_source(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
    }
}
