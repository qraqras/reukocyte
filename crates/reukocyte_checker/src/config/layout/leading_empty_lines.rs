use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Layout/LeadingEmptyLines.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct LeadingEmptyLinesConfig {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
}
impl Default for LeadingEmptyLinesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
        }
    }
}
