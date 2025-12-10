use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/IndentationWidth.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationWidth {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    pub width: i32,
    pub allowed_patterns: Vec<i32>,
}
impl Default for IndentationWidth {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            width: 2,
            allowed_patterns: Vec::new(),
        }
    }
}
