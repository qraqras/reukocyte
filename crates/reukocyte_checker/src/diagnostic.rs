//! Diagnostic type for reporting violations.

/// A diagnostic message for a rule violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The rule name (e.g., "Layout/TrailingWhitespace", "Lint/Debugger")
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
