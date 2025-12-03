pub mod rules;

/// A diagnostic message for a layout violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The rule name (e.g., "Layout/TrailingWhitespace")
    pub rule: &'static str,
    /// The message describing the violation
    pub message: String,
    /// Start byte offset in the source
    pub start: usize,
    /// End byte offset in the source
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// Check a Ruby source file for layout violations.
///
/// Note: This is kept for backwards compatibility and testing.
/// The main entry point is now `reukocyte_checker::check()`.
pub fn check(source: &[u8]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Run all layout rules
    rules::trailing_whitespace::check_source(source, |d| diagnostics.push(d));

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_no_violations() {
        let source = b"def foo\n  bar\nend\n";
        let diagnostics = check(source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_check_with_violations() {
        let source = b"def foo  \n  bar\nend\n";
        let diagnostics = check(source);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "Layout/TrailingWhitespace");
    }
}
