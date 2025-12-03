use crate::rule::RuleId;

/// Diagnostic information for a code issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub rule_id: RuleId,     // Rule identifier (typed)
    pub message: String,     // Description of the issue
    pub severity: Severity,  // Severity level of the issue
    pub start: usize,        // Start byte offset
    pub end: usize,          // End byte offset
    pub line_start: usize,   // Start line number
    pub line_end: usize,     // End line number
    pub column_start: usize, // Start column number
    pub column_end: usize,   // End column number
    pub fix: Option<Fix>,    // Optional fix for the issue
}
impl Diagnostic {
    /// Create a new diagnostic.
    pub fn new(
        rule_id: RuleId,
        message: String,
        severity: Severity,
        start: usize,
        end: usize,
        line_start: usize,
        line_end: usize,
        column_start: usize,
        column_end: usize,
        fix: Option<Fix>,
    ) -> Self {
        Self {
            rule_id,
            message,
            severity,
            start,
            end,
            line_start,
            line_end,
            column_start,
            column_end,
            fix,
        }
    }
    /// Get the rule name as a string (for API compatibility).
    pub fn rule(&self) -> String {
        format!(
            "{}/{}",
            self.rule_id.category().as_str(),
            self.rule_id.name()
        )
    }
    /// Set a fix for the diagnostic.
    pub fn set_fix(&mut self, fix: Fix) {
        self.fix = Some(fix);
    }
    /// Check if the diagnostic has a fix.
    pub fn correctable(&self) -> bool {
        self.fix.is_some()
    }
    /// Get the length of the diagnostic range.
    pub fn length(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    Refactor,
    #[default]
    Convention,
    Warning,
    Error,
    Fatal,
}
impl Severity {
    /// Get the string representation of the severity.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Refactor => "refactor",
            Severity::Convention => "convention",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Fatal => "fatal",
        }
    }
}

/// Applicability level for a fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Applicability {
    Safe,
    Unsafe,
    DisplayOnly,
}

/// A fix for a diagnostic, consisting of one or more edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub applicability: Applicability,
    pub edits: Vec<Edit>,
}
impl Fix {
    /// Create a safe fix.
    pub fn safe(edits: Vec<Edit>) -> Self {
        Self {
            applicability: Applicability::Safe,
            edits,
        }
    }
    /// Create an unsafe fix.
    pub fn r#unsafe(edits: Vec<Edit>) -> Self {
        Self {
            applicability: Applicability::Unsafe,
            edits,
        }
    }
    /// Create a display-only fix.
    pub fn display_only(edits: Vec<Edit>) -> Self {
        Self {
            applicability: Applicability::DisplayOnly,
            edits,
        }
    }
}

/// A single text edit (replacement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub content: String,
}
impl Edit {
    /// Create a new edit that replaces a range with new content.
    pub fn replacement(start: usize, end: usize, content: String) -> Self {
        Self {
            start,
            end,
            content,
        }
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
