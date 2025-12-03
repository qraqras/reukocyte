//! Diagnostic type for reporting violations.
//!
//! Inspired by Ruff's design:
//! - `Diagnostic`: The main violation report
//! - `Fix`: A set of edits to fix the violation
//! - `Edit`: A single text replacement
//! - `Applicability`: Safety level of the fix

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
    /// Optional fix for the violation
    pub fix: Option<Fix>,
}

impl Diagnostic {
    /// Create a new diagnostic without a fix.
    pub fn new(
        rule: &'static str,
        message: String,
        start: usize,
        end: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            rule,
            message,
            start,
            end,
            line,
            column,
            fix: None,
        }
    }

    /// Create a new diagnostic with a fix.
    pub fn with_fix(
        rule: &'static str,
        message: String,
        start: usize,
        end: usize,
        line: usize,
        column: usize,
        fix: Fix,
    ) -> Self {
        Self {
            rule,
            message,
            start,
            end,
            line,
            column,
            fix: Some(fix),
        }
    }

    /// Set the fix for this diagnostic.
    pub fn set_fix(&mut self, fix: Fix) {
        self.fix = Some(fix);
    }
}

/// A fix for a diagnostic, consisting of one or more edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// The edits that make up this fix
    pub edits: Vec<Edit>,
    /// The applicability (safety level) of this fix
    pub applicability: Applicability,
}

impl Fix {
    /// Create a safe fix (always safe to apply automatically).
    pub fn safe(edits: Vec<Edit>) -> Self {
        Self {
            edits,
            applicability: Applicability::Safe,
        }
    }

    /// Create an unsafe fix (may change code semantics).
    pub fn r#unsafe(edits: Vec<Edit>) -> Self {
        Self {
            edits,
            applicability: Applicability::Unsafe,
        }
    }

    /// Create a display-only fix (for informational purposes only).
    pub fn display_only(edits: Vec<Edit>) -> Self {
        Self {
            edits,
            applicability: Applicability::DisplayOnly,
        }
    }

    /// Create a safe fix from a single edit.
    pub fn safe_edit(edit: Edit) -> Self {
        Self::safe(vec![edit])
    }

    /// Create an unsafe fix from a single edit.
    pub fn unsafe_edit(edit: Edit) -> Self {
        Self::r#unsafe(vec![edit])
    }
}

/// A single text edit (replacement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Replacement content (empty string for deletion)
    pub content: String,
}

impl Edit {
    /// Create a new edit that replaces a range with new content.
    pub fn replacement(start: usize, end: usize, content: String) -> Self {
        Self { start, end, content }
    }

    /// Create an edit that deletes a range.
    pub fn deletion(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            content: String::new(),
        }
    }

    /// Create an edit that inserts content at a position.
    pub fn insertion(position: usize, content: String) -> Self {
        Self {
            start: position,
            end: position,
            content,
        }
    }
}

/// The applicability (safety level) of a fix.
///
/// This determines whether a fix can be applied automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Applicability {
    /// The fix is always safe to apply automatically.
    /// It will not change the semantics of the code.
    Safe,
    /// The fix may change the semantics of the code.
    /// It should only be applied with user confirmation.
    Unsafe,
    /// The fix is for display purposes only.
    /// It should not be applied automatically.
    DisplayOnly,
}
