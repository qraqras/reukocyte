use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Lint/Debugger.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Debugger {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
}

impl Default for Debugger {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Warning,
        }
    }
}
