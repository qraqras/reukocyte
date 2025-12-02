//! Offense representation

/// Represents a single offense (violation) found by a cop
#[derive(Debug, Clone)]
pub struct Offense {
    /// The cop that found this offense (e.g., "Layout/TrailingWhitespace")
    pub cop_name: String,

    /// The message describing the offense
    pub message: String,

    /// The file path where the offense was found
    pub file_path: String,

    /// The location of the offense
    pub location: Location,

    /// Severity of the offense
    pub severity: Severity,
}

/// Location of an offense in the source code
#[derive(Debug, Clone)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Length of the offense in characters
    pub length: usize,
}

impl Location {
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        Self {
            line,
            column,
            length,
        }
    }
}

/// Severity level of an offense
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Convention,
    Warning,
    Error,
    Fatal,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "I"),
            Severity::Convention => write!(f, "C"),
            Severity::Warning => write!(f, "W"),
            Severity::Error => write!(f, "E"),
            Severity::Fatal => write!(f, "F"),
        }
    }
}

impl Offense {
    pub fn new(
        cop_name: impl Into<String>,
        message: impl Into<String>,
        file_path: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            cop_name: cop_name.into(),
            message: message.into(),
            file_path: file_path.into(),
            location,
            severity: Severity::Convention,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}

impl std::fmt::Display for Offense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {} {}",
            self.file_path,
            self.location.line,
            self.location.column,
            self.severity,
            self.cop_name,
            self.message
        )
    }
}
